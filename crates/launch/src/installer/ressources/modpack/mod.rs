// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Modpack pipeline: downloads archive, parses manifest, produces `Vec<Mods>`.

#![cfg(any(feature = "modrinth", feature = "curseforge"))]

mod overrides;
mod pipeline;

#[cfg(feature = "modrinth")]
mod modrinth;

#[cfg(feature = "curseforge")]
mod curseforge;

pub(crate) use pipeline::process;
