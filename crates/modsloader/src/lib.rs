// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Mod-source clients (Modrinth, CurseForge) and a BFS dependency resolver.

pub mod request;
pub mod with_mods;
pub mod instance_cache;

#[cfg(any(feature = "modrinth", feature = "curseforge"))]
pub mod resolver;

#[cfg(any(feature = "modrinth", feature = "curseforge"))]
pub mod modpack;

#[cfg(feature = "modrinth")]
pub mod modrinth;

#[cfg(feature = "curseforge")]
pub mod curseforge;

pub use request::{ModKey, ModRequest, ModSource};
pub use with_mods::WithMods;
pub use instance_cache::InstanceCache;

#[cfg(any(feature = "modrinth", feature = "curseforge"))]
pub use modpack::ModpackSource;
