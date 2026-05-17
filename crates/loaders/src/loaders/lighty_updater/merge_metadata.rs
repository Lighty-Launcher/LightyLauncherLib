use crate::types::version_metadata::{Version, VersionMetaData};
use crate::types::{Loader, LoaderExtensions, VersionInfo};
use lighty_core::QueryError;

pub type Result<T> = std::result::Result<T, QueryError>;

/// Fetches and merges the base loader's metadata into a [`Version`].
///
/// The base loader is selected from the `loader` string supplied by
/// `ServerInfo` (`"vanilla"`, `"fabric"`, `"quilt"`, `"neoforge"`, `"forge"`).
pub async fn merge_metadata<V: VersionInfo>(version: &V, loader: &str) -> Result<Version> {
    let loader_type = match loader {
        "vanilla" => Loader::Vanilla,
        "fabric" => Loader::Fabric,
        "quilt" => Loader::Quilt,
        "neoforge" => Loader::NeoForge,
        "forge" => Loader::Forge,
        _ => {
            return Err(QueryError::UnsupportedLoader(format!(
                "Unknown loader '{}' - please check your LightyUpdater config",
                loader
            )))
        }
    };

    let temp_version = TempVersionInfo {
        name: version.name().to_string(),
        loader_version: version.loader_version().to_string(),
        minecraft_version: version.minecraft_version().to_string(),
        game_dirs: version.game_dirs().to_path_buf(),
        java_dirs: version.java_dirs().to_path_buf(),
        loader: loader_type,
    };

    let metadata = temp_version.get_metadata().await?;

    let merged_metadata = match &*metadata {
        VersionMetaData::Version(version) => version.clone(),
        _ => {
            return Err(QueryError::UnsupportedLoader(
                "Failed to extract Version from metadata".to_string(),
            ))
        }
    };

    Ok(merged_metadata)
}

#[derive(Clone)]
struct TempVersionInfo {
    name: String,
    loader_version: String,
    minecraft_version: String,
    game_dirs: std::path::PathBuf,
    java_dirs: std::path::PathBuf,
    loader: Loader,
}

impl VersionInfo for TempVersionInfo {
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

    fn game_dirs(&self) -> &std::path::Path {
        &self.game_dirs
    }

    fn java_dirs(&self) -> &std::path::Path {
        &self.java_dirs
    }

    fn loader(&self) -> &Self::LoaderType {
        &self.loader
    }
}
