// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Resource-installer steps: libraries, natives, client JAR, assets,
//! and partitioned mod-like assets (mods, resourcepacks, shaderpacks,
//! datapacks).

mod asset_partition;

pub(crate) mod libraries;
pub(crate) mod mods;
pub(crate) mod resourcepacks;
pub(crate) mod shaderpacks;
pub(crate) mod datapacks;
pub(crate) mod natives;
pub(crate) mod client;
pub(crate) mod assets;
#[cfg(any(feature = "modrinth", feature = "curseforge"))]
pub(crate) mod modpack;
