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

/// Maven repository for NeoForge artifacts.
pub const NEOFORGE_MAVEN: &str = "https://maven.neoforged.net/releases";
/// Subdirectory under `libraries/` used to cache files extracted from
/// the NeoForge installer JAR (keeps Forge / NeoForge extracts isolated).
pub const NEOFORGE_EXTRACT_SUBDIR: &str = "net/neoforged";

pub type Result<T> = std::result::Result<T, QueryError>;

/// Shared cached repository for NeoForge manifests.
pub static NEOFORGE: Lazy<ManifestRepository<NeoForgeQuery>> = Lazy::new(|| ManifestRepository::new());

/// Sub-queries supported by the NeoForge loader.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NeoForgeQuery {
    Libraries,
    Arguments,
    MainClass,
    NeoForgeBuilder,
}

#[async_trait]
impl Query for NeoForgeQuery {
    type Query = NeoForgeQuery;
    type Data = VersionMetaData;
    type Raw = ForgeInstallProfile ;

    fn name() -> &'static str {
        "neoforge"
    }

    async fn fetch_full_data<V: VersionInfo>(version: &V) -> Result<ForgeInstallProfile> {
        let installer_url = build_installer_url(version);

        lighty_core::trace_debug!(url = %installer_url, loader = "neoforge", "Installer URL constructed");

        let profiles_dir = version.game_dirs().join(".neoforge");
        mkdir!(profiles_dir);

        let installer_path = profiles_dir.join(format!("neoforge-{}-installer.jar", version.loader_version()));

        let needs_download = if installer_path.exists() {
            match verify_installer_sha1(&installer_path, &installer_url).await {
                Ok(true) => {
                    lighty_core::trace_info!(loader = "neoforge", "Installer already exists and SHA1 is valid");
                    false
                }
                Ok(false) => {
                    lighty_core::trace_warn!(loader = "neoforge", "Installer exists but SHA1 mismatch, re-downloading");
                    true
                }
                Err(_e) => {
                    lighty_core::trace_warn!(error = %_e, loader = "neoforge", "Could not verify SHA1, using existing file");
                    false
                }
            }
        } else {
            true
        };

        if needs_download {
            lighty_core::trace_info!(path = ?installer_path, loader = "neoforge", "Downloading installer");
            download_file_untracked(&installer_url, &installer_path)
                .await
                .map_err(|e| QueryError::Conversion {
                    message: format!("Failed to download installer: {}", e)
                })?;

            if let Ok(valid) = verify_installer_sha1(&installer_path, &installer_url).await {
                if !valid {
                    return Err(QueryError::Conversion {
                        message: "Downloaded installer has invalid SHA1".to_string()
                    });
                }
            }
        }

        let (install_profile, _) = read_jsons_from_jar(&installer_path).await?;

        lighty_core::trace_info!(loader = "neoforge", "Successfully loaded NeoForge metadata");

        Ok(install_profile)
    }

    async fn extract<V: VersionInfo>(version: &V, query: &Self::Query, full_data: &ForgeInstallProfile) -> Result<Self::Data> {
        let result = match query {
            NeoForgeQuery::Libraries => VersionMetaData::Libraries(extract_install_profile_libraries(full_data)),
            &NeoForgeQuery::Arguments | &NeoForgeQuery::MainClass => todo!(),
            NeoForgeQuery::NeoForgeBuilder => {
                VersionMetaData::Version(Self::version_builder(version, full_data).await?)
            }
        };
        Ok(result)
    }

    async fn version_builder<V: VersionInfo>(version: &V, _full_data: &ForgeInstallProfile) -> Result<Version> {
        let (vanilla_builder, version_meta) = tokio::try_join!(
            async {
                let vanilla_data = VanillaQuery::fetch_full_data(version).await?;
                VanillaQuery::version_builder(version, &vanilla_data).await
            },
            async {
                let profiles_dir = version.game_dirs().join(".neoforge");
                let installer_path = profiles_dir.join(format!("neoforge-{}-installer.jar", version.loader_version()));
                let (_, version_meta) = read_jsons_from_jar(&installer_path).await?;
                Ok::<_, QueryError>(version_meta)
            }
        )?;

        // Use ONLY runtime libraries from version.json. install_profile
        // libraries are processor-only and must not end up on the classpath.
        let version_json_libs = extract_libraries_from_version_meta(&version_meta);

        let merged_libs = merge_libraries(vanilla_builder.libraries, version_json_libs);

        Ok(Version {
            main_class: merge_main_class(vanilla_builder.main_class, extract_main_class(&version_meta)),
            java_version: vanilla_builder.java_version,
            arguments: merge_arguments(vanilla_builder.arguments, extract_arguments(&version_meta)),
            libraries: merged_libs,
            mods: None,
            natives: vanilla_builder.natives,
            client: vanilla_builder.client,
            assets_index: vanilla_builder.assets_index,
            assets: vanilla_builder.assets,
        })
    }
}

fn merge_main_class(vanilla: MainClass, neoforge: MainClass) -> MainClass {
    if neoforge.main_class.is_empty() {
        vanilla
    } else {
        neoforge
    }
}

fn merge_arguments(vanilla: Arguments, neoforge: Arguments) -> Arguments {
    Arguments {
        game: {
            let mut args = vanilla.game;
            args.extend(neoforge.game);
            args
        },
        jvm: match (vanilla.jvm, neoforge.jvm) {
            (Some(mut v), Some(n)) => {
                v.extend(n);
                Some(v)
            }
            (Some(v), None) => Some(v),
            (None, Some(n)) => Some(n),
            (None, None) => None,
        },
    }
}

/// Merges library lists, de-duplicating by `group:artifact[:classifier]`.
fn merge_libraries(vanilla_libs: Vec<Library>, neoforge_libs: Vec<Library>) -> Vec<Library> {
    let capacity = vanilla_libs.len() + neoforge_libs.len();
    let mut lib_map: HashMap<String, Library> = HashMap::with_capacity(capacity);

    for lib in vanilla_libs {
        let key = extract_artifact_key(&lib.name);
        lib_map.insert(key, lib);
    }

    // NeoForge overrides Vanilla on key collision (typically newer version).
    for lib in neoforge_libs {
        let key = extract_artifact_key(&lib.name);
        lib_map.insert(key, lib);
    }

    lib_map.into_values().collect()
}

/// Extracts the `group:artifact[:classifier]` key used for dedup.
/// The classifier MUST stay in the key — NeoForge's version.json ships
/// two libs with the same coords differing only by classifier
/// (`:universal` and `:client`); collapsing them would drop the FML
/// system mod and crash with "Failed to find system mod".
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
    // NeoForge always ships the structured `arguments` block; the Option
    // here is forced by the shared `ForgeVersionManifest` schema (which
    // also serves back-ported modern Forge installers).
    if let Some(args) = &version_meta.arguments {
        return Arguments {
            game: args.game.clone(),
            jvm: Some(args.jvm.clone()),
        };
    }
    Arguments { game: Vec::new(), jvm: None }
}

/// Returns the install_profile.json libraries as the launcher's pivot
/// [`Library`] type.
///
/// Includes both the processor JARs and the runtime-required artifacts
/// (notably `net.neoforged:forge:VERSION:universal`, referenced at
/// runtime by FML but absent from `version.json`).
pub fn extract_install_profile_libraries(full_data: &ForgeInstallProfile) -> Vec<Library> {
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

fn is_old_neoforge<V: VersionInfo>(version: &V) -> bool {
    version_compare::compare_to(version.minecraft_version(), "1.20.1", version_compare::Cmp::Le)
        .unwrap_or(false)
}

/// Builds the Maven URL of the NeoForge installer JAR for `version`.
pub fn build_installer_url<V: VersionInfo>(version: &V) -> String {
    if is_old_neoforge(version) {
        let path_version = format!("{}-{}", version.minecraft_version(), version.loader_version());
        let file_prefix = format!("forge-{}", version.minecraft_version());
        format!(
            "{}/{}/forge/{}/{}-{}-installer.jar",
            NEOFORGE_MAVEN,
            NEOFORGE_EXTRACT_SUBDIR,
            path_version,
            file_prefix,
            version.loader_version()
        )
    } else {
        format!(
            "{}/{}/neoforge/{}/neoforge-{}-installer.jar",
            NEOFORGE_MAVEN,
            NEOFORGE_EXTRACT_SUBDIR,
            version.loader_version(),
            version.loader_version()
        )
    }
}

/// Returns the on-disk path where the NeoForge installer is cached.
pub fn installer_cache_path<V: VersionInfo>(version: &V) -> PathBuf {
    version
        .game_dirs()
        .join(".neoforge")
        .join(format!("neoforge-{}-installer.jar", version.loader_version()))
}

/// Reads `install_profile.json` and `version.json` directly from the
/// installer JAR without extracting anything to disk.
async fn read_jsons_from_jar(installer_path: &PathBuf) -> Result<(ForgeInstallProfile, ForgeVersionManifest)> {
    let path = installer_path.clone();

    tokio::task::spawn_blocking(move || {
        let file = File::open(&path).map_err(|e| QueryError::Conversion {
            message: format!("Failed to open installer JAR: {}", e)
        })?;

        let mut archive = ZipArchive::new(file).map_err(|e| QueryError::Conversion {
            message: format!("Failed to open ZIP archive: {}", e)
        })?;

        let install_profile = {
            let mut file = archive.by_name("install_profile.json").map_err(|_| {
                QueryError::MissingField {
                    field: "install_profile.json in installer JAR".to_string(),
                }
            })?;

            let mut contents = String::new();
            file.read_to_string(&mut contents).map_err(|e| QueryError::Conversion {
                message: format!("Failed to read install_profile.json: {}", e)
            })?;

            serde_json::from_str::<ForgeInstallProfile>(&contents)?
        };

        let version_meta = {
            let mut file = archive.by_name("version.json").map_err(|_| {
                QueryError::MissingField {
                    field: "version.json in installer JAR".to_string(),
                }
            })?;

            let mut contents = String::new();
            file.read_to_string(&mut contents).map_err(|e| QueryError::Conversion {
                message: format!("Failed to read version.json: {}", e)
            })?;

            serde_json::from_str::<ForgeVersionManifest>(&contents)?
        };

        Ok((install_profile, version_meta))
    })
    .await
    .map_err(|e| QueryError::Conversion {
        message: format!("Failed to spawn blocking task: {}", e)
    })?
}

/// Verifies the local installer JAR matches the expected SHA1 from Maven.
async fn verify_installer_sha1(installer_path: &PathBuf, installer_url: &str) -> Result<bool> {
    let expected_sha1 = fetch_maven_sha1(installer_url)
        .await
        .ok_or_else(|| QueryError::Conversion {
            message: "Failed to fetch SHA1 from Maven".to_string()
        })?;

    lighty_core::verify_file_sha1_sync(installer_path, &expected_sha1)
        .map_err(|e| QueryError::Conversion {
            message: format!("Failed to verify SHA1: {}", e)
        })
}

