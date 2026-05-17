// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

pub mod installer;
pub mod config;
mod downloader;
mod verifier;

pub(crate) mod ressources;
// Forge/NeoForge install-processor pipeline (spawns Java; lives in `launch`
// rather than `loaders` to share the resolved JRE path).
#[cfg(any(feature = "forge", feature = "neoforge"))]
pub(crate) mod processors;

pub use installer::Installer;
