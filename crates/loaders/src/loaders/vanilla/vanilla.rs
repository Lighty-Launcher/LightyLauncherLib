use async_trait::async_trait;
use once_cell::sync::Lazy;
use lighty_core::{mkdir, verify_file_sha1};
use lighty_core::system::{ARCHITECTURE, OS};
use lighty_core::QueryError;
use crate::utils::manifest::ManifestRepository;
use crate::utils::query::Query;
use super::vanilla_metadata::{PistonMetaManifest, VanillaAssetFile,VanillaMetaData,Rule};
use crate::types::version_metadata::
{VersionMetaData,JavaVersion, Library, MainClass,Native,Client,AssetIndex,Asset, Arguments,
 Version, AssetsFile
};
use crate::types::VersionInfo;
use lighty_core::hosts::HTTP_CLIENT as CLIENT;

pub type Result<T> = std::result::Result<T, QueryError>;

const CLIENT_NAME: &str = "client";

/// Mojang's top-level version manifest (lists every released MC version).
const PISTON_META_MANIFEST_URL: &str =
    "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
/// Base URL for the asset CDN — assets live at `{prefix2}/{hash}`.
const MINECRAFT_RESOURCES: &str = "https://resources.download.minecraft.net";

/// Shared cached repository for Mojang's vanilla manifests.
pub static VANILLA: Lazy<ManifestRepository<VanillaQuery>> = Lazy::new(|| ManifestRepository::new());

/// Sub-queries supported by the Vanilla loader.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VanillaQuery {
    JavaVersion,
    MainClass,
    Libraries,
    Natives,
    AssetsIndex,
    Assets,
    Client,
    Arguments,
    VanillaBuilder,
}

#[async_trait]
impl Query for VanillaQuery {
    type Query = VanillaQuery;
    type Data = VersionMetaData;
    type Raw = VanillaMetaData;

    fn name() -> &'static str {
        "vanilla"
    }

    async fn fetch_full_data<V: VersionInfo>(version: &V) -> Result<VanillaMetaData> {
        lighty_core::trace_info!("Fetching manifest from {}", PISTON_META_MANIFEST_URL);

        let manifest: PistonMetaManifest =
            CLIENT.get(PISTON_META_MANIFEST_URL).send().await?.json().await?;

        let version_info = manifest
            .versions
            .iter()
            .find(|v| v.id == version.minecraft_version())
            .ok_or_else(|| QueryError::VersionNotFound {
                version: version.minecraft_version().to_string()
            })?;

        let vanilla_metadata: VanillaMetaData = CLIENT.get(&version_info.url).send().await?.json().await?;

        Ok(vanilla_metadata)
    }

    async fn extract<V: VersionInfo>(version: &V, query: &Self::Query, full_data: &Self::Raw) -> Result<Self::Data> {
        let result = match query {
            VanillaQuery::JavaVersion => VersionMetaData::JavaVersion(extract_java_version(full_data)),
            VanillaQuery::MainClass => VersionMetaData::MainClass(extract_main_class(full_data)),
            VanillaQuery::Libraries => VersionMetaData::Libraries(extract_libraries(full_data)),
            VanillaQuery::Natives => VersionMetaData::Natives(extract_natives(full_data)?),
            VanillaQuery::AssetsIndex => VersionMetaData::AssetsIndex(extract_assets_index(full_data)),
            VanillaQuery::Assets => VersionMetaData::Assets(extract_assets(version,full_data).await?),
            VanillaQuery::Client => VersionMetaData::Client(extract_client(version, full_data)?),
            VanillaQuery::Arguments => VersionMetaData::Arguments(extract_arguments(full_data)),
            VanillaQuery::VanillaBuilder => VersionMetaData::Version(Self::version_builder(version, full_data).await?),
        };

        Ok(result)
    }

    async fn version_builder<V: VersionInfo>(version: &V, full_data: &VanillaMetaData) -> Result<Version> {
        Ok(Version {
            main_class: extract_main_class(full_data),
            java_version: extract_java_version(full_data),
            arguments: extract_arguments(full_data),
            libraries: extract_libraries(full_data),
            mods: None,
            natives: Some(extract_natives(full_data)?),
            client: extract_client(version, full_data).ok(),
            assets_index: Some(extract_assets_index(full_data)),
            assets: Some(extract_assets(version, full_data).await?),
        })
    }
}

fn extract_libraries(full_data: &VanillaMetaData) -> Vec<Library> {
    full_data.libraries
        .iter()
        .filter(|lib| !lib.name.contains(":natives-") && lib.downloads.classifiers.is_none())
        .filter_map(|lib| {
            lib.downloads.artifact.as_ref().map(|a| Library {
                name: lib.name.clone(),
                url: Some(a.url.clone()),
                path: Some(a.path.clone()),
                sha1: Some(a.sha1.clone()),
                size: Some(a.size),
            })
        })
        .collect()
}

fn extract_natives(full_data: &VanillaMetaData) -> Result<Vec<Native>> {
    let os_name = OS.get_vanilla_os()
        .map_err(|_| QueryError::Conversion {
            message: format!("Unsupported operating system. Only Windows, Linux, and macOS are supported for native extraction. Detected OS: {:?}", std::env::consts::OS)
        })?;

    let arch_suffix = ARCHITECTURE.get_vanilla_arch()
        .map_err(|_| QueryError::Conversion {
            message: format!("Unsupported architecture. Only x86, x64, ARM, and ARM64 are supported. Detected architecture: {:?}", std::env::consts::ARCH)
        })?;

    let arch_bits = ARCHITECTURE.get_arch_bits()
        .map_err(|_| QueryError::Conversion {
            message: format!("Unable to determine architecture bits (32 or 64). Detected architecture: {:?}", std::env::consts::ARCH)
        })?;

    // Natives selection strategy:
    //
    // - In MC 1.19+ Mojang renamed the macOS classifier from "osx" to "macos"
    //   (LWJGL 3.3+), so we try both spellings when running on macOS.
    // - Pre-1.19 Minecraft only ships x64 macOS natives; on Apple Silicon
    //   we try the native arm64 classifier first and silently fall back to
    //   the x64 set, which the JVM can run under Rosetta 2.
    let os_names: Vec<&str> = if os_name == "osx" {
        vec!["osx", "macos"]
    } else {
        vec![os_name]
    };

    let arch_suffixes: Vec<&str> = if arch_suffix == "-arm64" && os_name == "osx" {
        vec!["-arm64", ""]
    } else {
        vec![arch_suffix]
    };

    let natives = full_data.libraries
        .iter()
        .filter_map(|lib| {
            // New format: `:natives-{os}{arch}` classifier.
            if lib.name.contains(":natives-") {
                for os in &os_names {
                    for arch in &arch_suffixes {
                        let exact_pattern = format!(":natives-{}{}", os, arch);

                        if lib.name.ends_with(&exact_pattern) {
                            if let Some(rules) = &lib.rules {
                                if !should_apply_rules(rules, os_name) {
                                    return None;
                                }
                            }

                            return lib.downloads.artifact.as_ref().map(|a| Native {
                                name: lib.name.clone(),
                                url: Some(a.url.clone()),
                                path: Some(a.path.clone()),
                                sha1: Some(a.sha1.clone()),
                                size: Some(a.size),
                            });
                        }
                    }
                }
            }

            // Legacy format: classifiers map.
            if let Some(natives_map) = &lib.natives {
                if let Some(classifiers) = &lib.downloads.classifiers {
                    if let Some(rules) = &lib.rules {
                        if !should_apply_rules(rules, os_name) {
                            return None;
                        }
                    }

                    for os in &os_names {
                        if let Some(classifier_pattern) = natives_map.get(*os) {
                            let classifier_name = classifier_pattern.replace("${arch}", arch_bits);

                            if let Some(artifact) = classifiers.get(&classifier_name) {
                                return Some(Native {
                                    name: lib.name.clone(),
                                    url: Some(artifact.url.clone()),
                                    path: Some(artifact.path.clone()),
                                    sha1: Some(artifact.sha1.clone()),
                                    size: Some(artifact.size),
                                });
                            }
                        }
                    }
                }
            }

            None
        })
        .collect();

    Ok(natives)
}

/// Returns whether the library's `rules` allow using it on the current OS.
pub(crate) fn should_apply_rules(rules: &[Rule], os_name: &str) -> bool {
    let mut allowed = false;

    for rule in rules {
        let matches_os = rule.os
            .as_ref()
            .map_or(true, |os| os.name.as_deref() == Some(os_name));

        if matches_os {
            allowed = rule.action == "allow";
        }
    }

    allowed
}

fn extract_java_version(full_data: &VanillaMetaData) -> JavaVersion {
    full_data.java_version
        .as_ref()
        .map(|v| JavaVersion { major_version: v.major_version as u8 })
        .unwrap_or_else(|| {
            // MC < 1.17 doesn't carry java_version; default to 8.
            JavaVersion { major_version: 8 }
        })
}

fn extract_main_class(full_data: &VanillaMetaData) -> MainClass {
    MainClass {
        main_class: full_data.main_class.clone(),
    }
}

fn extract_assets_index(full_data: &VanillaMetaData) -> AssetIndex {
    AssetIndex {
        id: full_data.asset_index.id.clone(),
        url: full_data.asset_index.url.clone(),
        sha1: full_data.asset_index.sha1.clone(),
        size: full_data.asset_index.size,
        total_size: full_data.asset_index.total_size,
    }
}

async fn extract_assets<V: VersionInfo>(version: &V, full_data: &VanillaMetaData) -> Result<AssetsFile> {
    let asset_index = &full_data.asset_index;

    let indexes_dir = version.game_dirs().join("assets").join("indexes");
    mkdir!(indexes_dir);

    let index_file_path = indexes_dir.join(format!("{}.json", asset_index.id));

    let needs_download = if index_file_path.exists() {
        match verify_file_sha1(&index_file_path, &asset_index.sha1).await {
            Ok(true) => {
                lighty_core::trace_info!("[Assets] Index {} already cached and valid", asset_index.id);
                false
            }
            _ => {
                lighty_core::trace_warn!("[Assets] Index {} SHA1 mismatch, re-downloading", asset_index.id);
                let _ = tokio::fs::remove_file(&index_file_path).await;
                true
            }
        }
    } else {
        true
    };

    if needs_download {
        lighty_core::trace_info!("[Assets] Downloading index {} from {}", asset_index.id, asset_index.url);

        let response = CLIENT.get(&asset_index.url).send().await?;

        if !response.status().is_success() {
            return Err(QueryError::Conversion {
                message: format!("Failed to download asset index: HTTP {}", response.status())
            });
        }

        let content = response.bytes().await?;


        tokio::fs::write(&index_file_path, &content).await?;


        match verify_file_sha1(&index_file_path, &asset_index.sha1).await {
            Ok(true) => {
                lighty_core::trace_info!("[Assets] Index {} downloaded and verified", asset_index.id);
            }
            _ => {
                let _ = tokio::fs::remove_file(&index_file_path).await;
                return Err(QueryError::Conversion {
                    message: format!("Downloaded asset index failed SHA1 verification")
                });
            }
        }
    }


    let index_content = tokio::fs::read_to_string(&index_file_path).await?;
    let vanilla_assets: VanillaAssetFile = serde_json::from_str(&index_content)?;

    let objects = vanilla_assets.objects
        .into_iter()
        .map(|(k, v)| {
            let url = Some(format!(
                "{}/{}/{}",
                MINECRAFT_RESOURCES,
                &v.hash[0..2],
                v.hash
            ));
            (k, Asset {
                hash: v.hash,
                size: v.size,
                url,
                path: None,
            })
        })
        .collect();

    Ok(AssetsFile { objects })
}

fn extract_client<V: VersionInfo>(version: &V, full_data: &VanillaMetaData) -> Result<Client> {
    full_data.downloads.client
        .as_ref()
        .map(|client| Client {
            name: CLIENT_NAME.into(),
            url: Some(client.url.clone()),
            path: Some(format!("{}.jar", version.name())),
            sha1: Some(client.sha1.clone()),
            size: Some(client.size),
        })
        .ok_or_else(|| QueryError::MissingField {
            field: CLIENT_NAME.into(),
        })
}

fn extract_arguments(full_data: &VanillaMetaData) -> Arguments {
    if let Some(args) = &full_data.arguments {
        Arguments {
            game: args.game
                .iter()
                .filter_map(|a| a.as_str().map(String::from))
                .collect(),
            jvm: Some(
                args.jvm
                    .iter()
                    .filter_map(|a| a.as_str().map(String::from))
                    .collect(),
            ),
        }
    } else if let Some(legacy) = &full_data.minecraft_arguments {
        Arguments {
            game: legacy.split_whitespace().map(String::from).collect(),
            jvm: None,
        }
    } else {
        Arguments {
            game: vec![],
            jvm: None,
        }
    }
}