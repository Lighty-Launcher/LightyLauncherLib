//! Forge loader — modern (>= 1.13) and legacy (1.4 -> 1.12.2) dispatch.

use async_trait::async_trait;
use once_cell::sync::Lazy;
use std::{collections::HashMap, fs::File, io::Read, path::PathBuf};
use zip::ZipArchive;

use lighty_core::download::download_file_untracked;
use lighty_core::mkdir;

use crate::loaders::vanilla::vanilla::VanillaQuery;
use crate::types::version_metadata::{Arguments, Library, MainClass, Version, VersionMetaData};
use crate::types::VersionInfo;
use crate::utils::forge_installer::{ForgeInstallProfile, ForgeVersionManifest};
use crate::utils::maven::fetch_maven_sha1;
use lighty_core::QueryError;
use crate::utils::{manifest::ManifestRepository, query::Query};

use super::forge_legacy::{self, is_legacy_forge, InstallProfileKind};
use super::forge_legacy_metadata::ForgeLegacyInstallProfile;

/// Maven repository for Forge artifacts.
pub const FORGE_MAVEN: &str = "https://maven.minecraftforge.net";
/// Subdirectory under `libraries/` used to cache files extracted from
/// the Forge installer JAR.
pub const FORGE_EXTRACT_SUBDIR: &str = "net/minecraftforge";

pub type Result<T> = std::result::Result<T, QueryError>;

/// Shared cached repository for Forge manifests (covers both eras).
pub static FORGE: Lazy<ManifestRepository<ForgeQuery>> =
    Lazy::new(|| ManifestRepository::new());

/// Sub-queries supported by the Forge loader.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ForgeQuery {
    Libraries,
    ForgeBuilder,
}

/// Raw installer data — dispatched on the Minecraft version.
#[derive(Debug, Clone)]
pub enum ForgeRawData {
    Modern {
        install_profile: ForgeInstallProfile,
        version_manifest: ForgeVersionManifest,
    },
    Legacy(ForgeLegacyInstallProfile),
}

#[async_trait]
impl Query for ForgeQuery {
    type Query = ForgeQuery;
    type Data = VersionMetaData;
    type Raw = ForgeRawData;

    fn name() -> &'static str {
        "forge"
    }

    async fn fetch_full_data<V: VersionInfo>(version: &V) -> Result<ForgeRawData> {
        if !is_legacy_forge(version.minecraft_version()) {
            return fetch_modern_install_data(version).await;
        }

        // MC < 1.13: the URL pattern follows legacy rules but the
        // install_profile.json inside may be either schema — late 1.12.2
        // builds (e.g. 14.23.5.2860) ship the modern schema. Download
        // once, then dispatch on actual content.
        let installer_path = forge_legacy::ensure_installer_cached(version).await?;
        match forge_legacy::peek_install_profile_kind(&installer_path).await? {
            InstallProfileKind::Legacy => {
                let profile =
                    forge_legacy::read_install_profile_from_jar(&installer_path).await?;
                Ok(ForgeRawData::Legacy(profile))
            }
            InstallProfileKind::Modern => {
                let (install_profile, version_manifest) =
                    read_jsons_from_jar(&installer_path).await?;
                Ok(ForgeRawData::Modern {
                    install_profile,
                    version_manifest,
                })
            }
        }
    }

    async fn extract<V: VersionInfo>(
        version: &V,
        query: &Self::Query,
        full_data: &ForgeRawData,
    ) -> Result<Self::Data> {
        let result = match (query, full_data) {
            (ForgeQuery::Libraries, ForgeRawData::Modern { install_profile, .. }) => {
                VersionMetaData::Libraries(extract_install_profile_libraries_modern(
                    install_profile,
                ))
            }
            (ForgeQuery::Libraries, ForgeRawData::Legacy(profile)) => {
                VersionMetaData::Libraries(forge_legacy::extract_legacy_libraries(profile).await)
            }
            (ForgeQuery::ForgeBuilder, _) => {
                VersionMetaData::Version(Self::version_builder(version, full_data).await?)
            }
        };
        Ok(result)
    }

    async fn version_builder<V: VersionInfo>(
        version: &V,
        full_data: &ForgeRawData,
    ) -> Result<Version> {
        match full_data {
            ForgeRawData::Modern { version_manifest, .. } => {
                modern_version_builder(version, version_manifest).await
            }
            ForgeRawData::Legacy(profile) => {
                forge_legacy::legacy_version_builder(version, profile).await
            }
        }
    }
}

async fn fetch_modern_install_data<V: VersionInfo>(version: &V) -> Result<ForgeRawData> {
    let installer_url = build_installer_url(version);
    lighty_core::trace_debug!(url = %installer_url, loader = "forge", "Installer URL constructed");

    let profiles_dir = version.game_dirs().join(".forge");
    mkdir!(profiles_dir);

    let installer_path = installer_cache_path(version);

    let needs_download = if installer_path.exists() {
        match verify_installer_sha1(&installer_path, &installer_url).await {
            Ok(true) => {
                lighty_core::trace_info!(loader = "forge", "Installer already exists and SHA1 is valid");
                false
            }
            Ok(false) => {
                lighty_core::trace_warn!(loader = "forge", "Installer exists but SHA1 mismatch, re-downloading");
                true
            }
            Err(_e) => {
                lighty_core::trace_warn!(error = %_e, loader = "forge", "Could not verify SHA1, using existing file");
                false
            }
        }
    } else {
        true
    };

    if needs_download {
        lighty_core::trace_info!(path = ?installer_path, loader = "forge", "Downloading installer");
        download_file_untracked(&installer_url, &installer_path)
            .await
            .map_err(|e| QueryError::Conversion {
                message: format!("Failed to download installer: {}", e),
            })?;

        if let Ok(valid) = verify_installer_sha1(&installer_path, &installer_url).await {
            if !valid {
                return Err(QueryError::Conversion {
                    message: "Downloaded installer has invalid SHA1".to_string(),
                });
            }
        }
    }

    let (install_profile, version_manifest) = read_jsons_from_jar(&installer_path).await?;

    lighty_core::trace_info!(loader = "forge", "Successfully loaded modern Forge metadata");

    Ok(ForgeRawData::Modern {
        install_profile,
        version_manifest,
    })
}

/// Modern (>= 1.13) `Version` builder — merges vanilla baseline with
/// the Forge `version.json` overrides.
async fn modern_version_builder<V: VersionInfo>(
    version: &V,
    version_meta: &ForgeVersionManifest,
) -> Result<Version> {
    let vanilla_data = VanillaQuery::fetch_full_data(version).await?;
    let vanilla_builder = VanillaQuery::version_builder(version, &vanilla_data).await?;

    // Use ONLY runtime libraries from version.json. install_profile libraries
    // are processor-only and must not end up on the classpath.
    let version_json_libs = extract_libraries_from_version_meta(version_meta);

    let merged_libs = merge_libraries(vanilla_builder.libraries, version_json_libs);

    // Back-ported modern installers (MC < 1.13 with the modern schema,
    // e.g. Forge 14.23.5.2860 for 1.12.2) ship a `minecraftArguments`
    // string that already includes the full game-args line — vanilla's
    // `game` would duplicate flags like `--gameDir`. Forge replaces vanilla
    // in that case.
    let merged_arguments = if version_meta.minecraft_arguments.is_some() {
        replace_game_keep_jvm(vanilla_builder.arguments, extract_arguments(version_meta))
    } else {
        merge_arguments(vanilla_builder.arguments, extract_arguments(version_meta))
    };

    Ok(Version {
        main_class: merge_main_class(vanilla_builder.main_class, extract_main_class(version_meta)),
        java_version: vanilla_builder.java_version,
        arguments: merged_arguments,
        libraries: merged_libs,
        mods: None,
        natives: vanilla_builder.natives,
        client: vanilla_builder.client,
        assets_index: vanilla_builder.assets_index,
        assets: vanilla_builder.assets,
    })
}

/// Forge's `minecraftArguments` is a complete game-args line so vanilla's
/// `game` is discarded. JVM args are inherited from vanilla because the
/// back-ported version.json never carries any.
fn replace_game_keep_jvm(vanilla: Arguments, forge: Arguments) -> Arguments {
    Arguments {
        game: forge.game,
        jvm: vanilla.jvm,
    }
}

fn merge_main_class(vanilla: MainClass, forge: MainClass) -> MainClass {
    if forge.main_class.is_empty() {
        vanilla
    } else {
        forge
    }
}

fn merge_arguments(vanilla: Arguments, forge: Arguments) -> Arguments {
    Arguments {
        game: {
            let mut args = vanilla.game;
            args.extend(forge.game);
            args
        },
        jvm: match (vanilla.jvm, forge.jvm) {
            (Some(mut v), Some(f)) => {
                v.extend(f);
                Some(v)
            }
            (Some(v), None) => Some(v),
            (None, Some(f)) => Some(f),
            (None, None) => None,
        },
    }
}

/// Merges library lists, de-duplicating by `group:artifact[:classifier]`.
fn merge_libraries(vanilla_libs: Vec<Library>, forge_libs: Vec<Library>) -> Vec<Library> {
    let capacity = vanilla_libs.len() + forge_libs.len();
    let mut lib_map: HashMap<String, Library> = HashMap::with_capacity(capacity);

    for lib in vanilla_libs {
        let key = extract_artifact_key(&lib.name);
        lib_map.insert(key, lib);
    }

    // Forge overrides Vanilla on key collision (typically newer version).
    for lib in forge_libs {
        let key = extract_artifact_key(&lib.name);
        lib_map.insert(key, lib);
    }

    lib_map.into_values().collect()
}

/// Extracts the `group:artifact[:classifier]` key used for dedup.
/// The classifier MUST stay in the key — Forge's version.json ships two
/// libs with the same coordinates differing only by classifier
/// (`:universal` and `:client`); collapsing them would silently drop one
/// and Forge would crash with "Failed to find system mod: forge".
fn extract_artifact_key(maven_name: &str) -> String {
    let parts: Vec<&str> = maven_name.split(':').collect();
    match parts.as_slice() {
        [group, artifact, _version, classifier, ..] => {
            format!("{}:{}:{}", group, artifact, classifier)
        }
        [group, artifact, ..] => format!("{}:{}", group, artifact),
        _ => maven_name.to_string(),
    }
}

fn extract_main_class(version_meta: &ForgeVersionManifest) -> MainClass {
    MainClass {
        main_class: version_meta.main_class.clone(),
    }
}

fn extract_arguments(version_meta: &ForgeVersionManifest) -> Arguments {
    if let Some(args) = &version_meta.arguments {
        return Arguments {
            game: args.game.clone(),
            jvm: Some(args.jvm.clone()),
        };
    }
    // Back-ported modern installers keep the legacy single-string
    // `minecraftArguments`. JVM args are inherited from vanilla in that era.
    let game = version_meta
        .minecraft_arguments
        .as_deref()
        .map(|s| s.split_whitespace().map(String::from).collect())
        .unwrap_or_default();
    Arguments { game, jvm: None }
}

/// Returns the install_profile.json libraries (modern era, >= 1.13) as
/// the launcher's pivot [`Library`] type.
///
/// Includes both the processor JARs and the runtime-required artifacts
/// (notably `net.minecraftforge:forge:VERSION:universal`, referenced at
/// runtime by FML but absent from `version.json`).
pub fn extract_install_profile_libraries_modern(full_data: &ForgeInstallProfile) -> Vec<Library> {
    full_data
        .libraries
        .iter()
        .map(|lib| Library {
            name: lib.name.clone(),
            url: Some(lib.downloads.artifact.url.clone()),
            path: Some(lib.downloads.artifact.path.clone()),
            sha1: Some(lib.downloads.artifact.sha1.clone()),
            size: Some(lib.downloads.artifact.size),
        })
        .collect()
}

fn extract_libraries_from_version_meta(version_meta: &ForgeVersionManifest) -> Vec<Library> {
    version_meta
        .libraries
        .iter()
        .map(|lib| Library {
            name: lib.name.clone(),
            url: Some(lib.downloads.artifact.url.clone()),
            path: Some(lib.downloads.artifact.path.clone()),
            sha1: Some(lib.downloads.artifact.sha1.clone()),
            size: Some(lib.downloads.artifact.size),
        })
        .collect()
}

/// Builds the Maven URL of the modern Forge installer JAR for `version`.
pub fn build_installer_url<V: VersionInfo>(version: &V) -> String {
    format!(
        "{}/net/minecraftforge/forge/{}-{}/forge-{}-{}-installer.jar",
        FORGE_MAVEN,
        version.minecraft_version(),
        version.loader_version(),
        version.minecraft_version(),
        version.loader_version(),
    )
}

/// Returns the on-disk path where the modern Forge installer is cached.
pub fn installer_cache_path<V: VersionInfo>(version: &V) -> PathBuf {
    version
        .game_dirs()
        .join(".forge")
        .join(format!("forge-{}-installer.jar", version.loader_version()))
}

/// Reads `install_profile.json` and `version.json` directly from the
/// installer JAR without extracting anything to disk.
async fn read_jsons_from_jar(
    installer_path: &PathBuf,
) -> Result<(ForgeInstallProfile, ForgeVersionManifest)> {
    let path = installer_path.clone();

    tokio::task::spawn_blocking(move || {
        let file = File::open(&path).map_err(|e| QueryError::Conversion {
            message: format!("Failed to open installer JAR: {}", e),
        })?;

        let mut archive = ZipArchive::new(file).map_err(|e| QueryError::Conversion {
            message: format!("Failed to open ZIP archive: {}", e),
        })?;

        let install_profile = {
            let mut file = archive.by_name("install_profile.json").map_err(|_| {
                QueryError::MissingField {
                    field: "install_profile.json in installer JAR".to_string(),
                }
            })?;

            let mut contents = String::new();
            file.read_to_string(&mut contents)
                .map_err(|e| QueryError::Conversion {
                    message: format!("Failed to read install_profile.json: {}", e),
                })?;

            serde_json::from_str::<ForgeInstallProfile>(&contents)?
        };

        let version_meta = {
            let mut file = archive
                .by_name("version.json")
                .map_err(|_| QueryError::MissingField {
                    field: "version.json in installer JAR".to_string(),
                })?;

            let mut contents = String::new();
            file.read_to_string(&mut contents)
                .map_err(|e| QueryError::Conversion {
                    message: format!("Failed to read version.json: {}", e),
                })?;

            serde_json::from_str::<ForgeVersionManifest>(&contents)?
        };

        Ok((install_profile, version_meta))
    })
    .await
    .map_err(|e| QueryError::Conversion {
        message: format!("Failed to spawn blocking task: {}", e),
    })?
}

/// Verifies the local installer JAR matches the expected SHA1 from Maven.
async fn verify_installer_sha1(installer_path: &PathBuf, installer_url: &str) -> Result<bool> {
    let expected_sha1 = fetch_maven_sha1(installer_url)
        .await
        .ok_or_else(|| QueryError::Conversion {
            message: "Failed to fetch SHA1 from Maven".to_string(),
        })?;

    lighty_core::verify_file_sha1_sync(installer_path, &expected_sha1).map_err(|e| {
        QueryError::Conversion {
            message: format!("Failed to verify SHA1: {}", e),
        }
    })
}
