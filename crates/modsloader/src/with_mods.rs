// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Trait carrying the user-attached mods and optional modpack for an instance.

use crate::request::ModRequest;

#[cfg(any(feature = "modrinth", feature = "curseforge"))]
use crate::modpack::ModpackSource;

/// Exposes the mod requests and optional modpack source for a builder.
pub trait WithMods {
    /// User-attached mod requests pulled from Modrinth / CurseForge at install time.
    fn mod_requests(&self) -> &[ModRequest];

    /// Optional modpack to install before the user-attached mods. The
    /// modpack's manifest is authoritative on the loader + Minecraft
    /// version; conflicts are logged and the modpack wins.
    #[cfg(any(feature = "modrinth", feature = "curseforge"))]
    fn modpack(&self) -> Option<&ModpackSource> {
        None
    }
}
