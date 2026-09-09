// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! The coordinates an instance really installs under.

use std::path::{Path, PathBuf};

use crate::types::{Loader, VersionInfo};

/// What a builder claims is enough for every loader but LightyUpdater,
/// whose builder holds a server URL where the loader version belongs and
/// no Minecraft version at all. Resolving once keeps the loader queries,
/// the installer paths and the cache keys pointed at the same instance.
#[derive(Debug, Clone)]
pub struct ResolvedInstance {
    name: String,
    loader: Loader,
    loader_version: String,
    minecraft_version: String,
    game_dirs: PathBuf,
    java_dirs: PathBuf,
}

impl ResolvedInstance {
    pub fn new(
        name: String,
        loader: Loader,
        loader_version: String,
        minecraft_version: String,
        game_dirs: PathBuf,
        java_dirs: PathBuf,
    ) -> Self {
        Self {
            name,
            loader,
            loader_version,
            minecraft_version,
            game_dirs,
            java_dirs,
        }
    }

    /// Copies a builder as-is, for the loaders that already tell the truth.
    pub fn of<V: VersionInfo<LoaderType = Loader>>(version: &V) -> Self {
        Self::new(
            version.name().to_string(),
            version.loader().clone(),
            version.loader_version().to_string(),
            version.minecraft_version().to_string(),
            version.game_dirs().to_path_buf(),
            version.java_dirs().to_path_buf(),
        )
    }
}

impl VersionInfo for ResolvedInstance {
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
