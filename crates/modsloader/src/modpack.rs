// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! User-facing description of where to fetch a modpack from.

/// Modpack to install into a [`VersionBuilder`](crate).
///
/// One per instance. Modpack files are installed before user `.with_*_mods(...)`
/// entries; on filename conflict the user mod wins.
#[derive(Debug, Clone)]
pub enum ModpackSource {
    #[cfg(feature = "modrinth")]
    ModrinthUrl(String),

    #[cfg(feature = "modrinth")]
    ModrinthPinned {
        project: String,
        version: Option<String>,
    },

    #[cfg(feature = "curseforge")]
    CurseForgePinned {
        project_id: u32,
        file_id: u32,
    },
}

#[cfg(feature = "modrinth")]
impl From<&str> for ModpackSource {
    fn from(url: &str) -> Self {
        Self::ModrinthUrl(url.to_string())
    }
}

#[cfg(feature = "modrinth")]
impl From<String> for ModpackSource {
    fn from(url: String) -> Self {
        Self::ModrinthUrl(url)
    }
}
