use crate::types::version_metadata::{Library, MainClass, Arguments, Version, VersionMetaData, JavaVersion, Mods, Native, Client, AssetsFile, Asset};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::types::{VersionInfo, Loader};
use lighty_core::QueryError;
use crate::utils::{query::Query, manifest::ManifestRepository};
use once_cell::sync::Lazy;
use super::lighty_metadata::{LightyMetadata, ServersResponse};
use async_trait::async_trait;
use lighty_core::hosts::HTTP_CLIENT as CLIENT;

pub type Result<T> = std::result::Result<T, QueryError>;

/// Internal `VersionInfo` view that swaps in the real loader and Minecraft
/// version sourced from `ServerInfo`.
#[derive(Debug, Clone)]
struct VersionOverride {
    name: String,
    loader_version: String,
    minecraft_version: String,
    loader: Loader,
    game_dirs: PathBuf,
    java_dirs: PathBuf,
}

impl VersionInfo for VersionOverride {
    type LoaderType = Loader;

    fn name(&self) -> &str {
        &self.name
    }

    fn loader_version(&self) -> &str {
        &self.loader_version
    }

    fn minecraft_version(&self) -> &str {
        &self.minecraft_version
    }

    fn game_dirs(&self) -> &Path {
        &self.game_dirs
    }

    fn java_dirs(&self) -> &Path {
        &self.java_dirs
    }

    fn loader(&self) -> &Self::LoaderType {
        &self.loader
    }
}

/// Shared cached repository for LightyUpdater server metadata.
pub static LIGHTY_UPDATER: Lazy<ManifestRepository<LightyQuery>> = Lazy::new(|| ManifestRepository::new());

/// Sub-queries supported by the LightyUpdater loader.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LightyQuery {
    Libraries,
    Arguments,
    MainClass,
    Mods,
    Assets,
    LightyBuilder,
}

#[async_trait]
impl Query for LightyQuery {
    type Query = LightyQuery;
    type Data = VersionMetaData;
    type Raw = LightyMetadata;

    fn name() -> &'static str {
        "lighty_updater"
    }

    async fn fetch_full_data<V: VersionInfo>(version: &V) -> Result<LightyMetadata> {
        let server_info_url = format!("{}/", version.loader_version());
        let response = CLIENT.get(&server_info_url).send().await?;
        let text = response.text().await?;

        let servers_response: ServersResponse = serde_json::from_str(&text)
            .map_err(QueryError::JsonParsing)?;

        let server_info = servers_response.find_by_name(version.name())
            .cloned()
            .ok_or_else(|| QueryError::VersionNotFound { version: version.name().to_string() })?;

        let metadata_url = server_info.url();
        let meta_response = CLIENT.get(metadata_url).send().await?;
        let mut manifest: LightyMetadata = meta_response.json().await?;

        // Store server_info on the metadata so version_builder doesn't refetch.
        manifest.server_info = Some(server_info);

        Ok(manifest)
    }

    async fn extract<V: VersionInfo>(version: &V, query: &Self::Query, full_data: &LightyMetadata) -> Result<Self::Data> {
        let result = match query {
            LightyQuery::Libraries => VersionMetaData::Libraries(extract_libraries(full_data)?),
            LightyQuery::Arguments => VersionMetaData::Arguments(extract_arguments(full_data)?),
            LightyQuery::MainClass => VersionMetaData::MainClass(extract_main_class(full_data)?),
            LightyQuery::Mods => VersionMetaData::Mods(extract_mods(full_data)?),
            LightyQuery::Assets => VersionMetaData::Assets(extract_assets(full_data)?),
            LightyQuery::LightyBuilder => VersionMetaData::Version(Self::version_builder(version, full_data).await?),
        };
        Ok(result)
    }

    async fn version_builder<V: VersionInfo>(version: &V, full_data: &LightyMetadata) -> Result<Version> {
        use super::merge_metadata::merge_metadata;

        let server_info = full_data.server_info.as_ref()
            .ok_or_else(|| QueryError::InvalidMetadata)?;

        let loader = match server_info.loader() {
            "vanilla" => Loader::Vanilla,
            "fabric" => Loader::Fabric,
            "quilt" => Loader::Quilt,
            "neoforge" => Loader::NeoForge,
            "forge" => Loader::Forge,
            _ => Loader::LightyUpdater,
        };

        let version_override = VersionOverride {
            name: version.name().to_string(),
            loader_version: server_info.loader_version().to_string(),
            minecraft_version: server_info.minecraft_version().to_string(),
            loader,
            game_dirs: version.game_dirs().to_path_buf(),
            java_dirs: version.java_dirs().to_path_buf(),
        };

        let mut builder = merge_metadata(&version_override, server_info.loader()).await?;

        // Apply LightyMetadata overrides on top (Lighty wins).

        if let Some(client) = &full_data.client {
            if !client.url.is_empty() {
                builder.client = Some(extract_client(full_data)?);
            }
        }

        if let Some(natives) = &full_data.natives {
            if !natives.is_empty() {
                let lighty_natives = extract_natives(full_data)?;
                builder.natives = Some(merge_natives(
                    builder.natives.unwrap_or_default(),
                    lighty_natives
                ));
            }
        }

        if let Some(assets) = &full_data.assets {
            if !assets.is_empty() {
                let lighty_assets = extract_assets(full_data)?;
                builder.assets = Some(merge_assets(
                    builder.assets.unwrap_or_else(|| AssetsFile { objects: HashMap::new() }),
                    lighty_assets
                ));
            }
        }

        if let Some(libraries) = &full_data.libraries {
            if !libraries.is_empty() {
                let lighty_libs = extract_libraries(full_data)?;
                builder.libraries = merge_libraries(builder.libraries, lighty_libs);
            }
        }

        if full_data.mods.is_some() {
            builder.mods = Some(extract_mods(full_data)?);
        }

        if let Some(main_class) = &full_data.main_class {
            if !main_class.main_class.is_empty() {
                builder.main_class = extract_main_class(full_data)?;
            }
        }

        if let Some(_args) = &full_data.arguments {
            builder.arguments = merge_arguments(builder.arguments, extract_arguments(full_data)?);
        }

        if let Some(java_version) = &full_data.java_version {
            if java_version.major_version > 0 {
                builder.java_version = extract_java_version(full_data)?;
            }
        }

        lighty_core::trace_info!(
            loader = %server_info.loader(),
            mods_count = builder.mods.as_ref().map(|m| m.len()).unwrap_or(0),
            "Merged Lighty Updater with {} loader",
            server_info.loader()
        );

        Ok(builder)
    }
}

// Each `extract_*` surfaces a structured `QueryError::MissingField` when
// the matching `Option<...>` is absent, instead of panicking.

fn extract_main_class(full_data: &LightyMetadata) -> Result<MainClass> {
    let mc = full_data.main_class.as_ref().ok_or_else(|| QueryError::MissingField {
        field: "lighty_updater.main_class".to_string(),
    })?;
    Ok(MainClass {
        main_class: mc.main_class.clone(),
    })
}

fn extract_java_version(full_data: &LightyMetadata) -> Result<JavaVersion> {
    let jv = full_data.java_version.as_ref().ok_or_else(|| QueryError::MissingField {
        field: "lighty_updater.java_version".to_string(),
    })?;
    Ok(JavaVersion {
        major_version: jv.major_version as u8,
    })
}

fn extract_arguments(full_data: &LightyMetadata) -> Result<Arguments> {
    let args = full_data.arguments.as_ref().ok_or_else(|| QueryError::MissingField {
        field: "lighty_updater.arguments".to_string(),
    })?;
    Ok(Arguments {
        game: args.game.clone(),
        // None falls back to the base loader's JVM args.
        jvm: if args.jvm.is_empty() {
            None
        } else {
            Some(args.jvm.clone())
        },
    })
}

fn extract_libraries(full_data: &LightyMetadata) -> Result<Vec<Library>> {
    let libs = full_data.libraries.as_ref().ok_or_else(|| QueryError::MissingField {
        field: "lighty_updater.libraries".to_string(),
    })?;
    Ok(libs.iter().map(|lib| Library {
        name: lib.name.clone(),
        url: lib.url.clone(),
        path: lib.path.clone(),
        sha1: lib.sha1.clone(),
        size: lib.size,
    }).collect())
}

fn extract_mods(full_data: &LightyMetadata) -> Result<Vec<Mods>> {
    let mods = full_data.mods.as_ref().ok_or_else(|| QueryError::MissingField {
        field: "lighty_updater.mods".to_string(),
    })?;
    Ok(mods.iter().map(|mod_| Mods {
        name: mod_.name.clone(),
        url: Some(mod_.url.clone()),
        path: Some(mod_.path.clone()),
        sha1: Some(mod_.sha1.clone()),
        size: Some(mod_.size),
    }).collect())
}

fn extract_natives(full_data: &LightyMetadata) -> Result<Vec<Native>> {
    let natives = full_data.natives.as_ref().ok_or_else(|| QueryError::MissingField {
        field: "lighty_updater.natives".to_string(),
    })?;
    Ok(natives.iter().map(|native| Native {
        name: native.name.clone(),
        url: Some(native.url.clone()),
        path: Some(native.path.clone()),
        sha1: Some(native.sha1.clone()),
        size: Some(native.size),
    }).collect())
}

fn extract_client(full_data: &LightyMetadata) -> Result<Client> {
    let client = full_data.client.as_ref().ok_or_else(|| QueryError::MissingField {
        field: "lighty_updater.client".to_string(),
    })?;
    Ok(Client {
        name: client.name.clone(),
        url: Some(client.url.clone()),
        path: Some(client.path.clone()),
        sha1: Some(client.sha1.clone()),
        size: Some(client.size),
    })
}

fn extract_assets(full_data: &LightyMetadata) -> Result<AssetsFile> {
    let assets = full_data.assets.as_ref().ok_or_else(|| QueryError::MissingField {
        field: "lighty_updater.assets".to_string(),
    })?;
    let mut objects = HashMap::new();

    for asset in assets {
        objects.insert(
            asset.hash.clone(),
            Asset {
                hash: asset.hash.clone(),
                size: asset.size,
                url: asset.url.clone(),
            }
        );
    }

    Ok(AssetsFile { objects })
}

fn merge_libraries(mut loader_libs: Vec<Library>, lighty_libs: Vec<Library>) -> Vec<Library> {
    loader_libs.extend(lighty_libs);
    loader_libs
}

fn merge_arguments(loader_args: Arguments, lighty_args: Arguments) -> Arguments {
    Arguments {
        game: {
            let mut args = loader_args.game;
            args.extend(lighty_args.game);
            args
        },
        jvm: match (loader_args.jvm, lighty_args.jvm) {
            (Some(mut loader_jvm), Some(lighty_jvm)) => {
                loader_jvm.extend(lighty_jvm);
                Some(loader_jvm)
            }
            (Some(loader_jvm), None) => Some(loader_jvm),
            (None, Some(lighty_jvm)) => Some(lighty_jvm),
            (None, None) => None,
        },
    }
}

fn merge_natives(mut loader_natives: Vec<Native>, lighty_natives: Vec<Native>) -> Vec<Native> {
    loader_natives.extend(lighty_natives);
    loader_natives
}

/// Merges asset maps; duplicate hashes are silently overwritten by Lighty.
fn merge_assets(mut loader_assets: AssetsFile, lighty_assets: AssetsFile) -> AssetsFile {
    loader_assets.objects.extend(lighty_assets.objects);
    loader_assets
}
