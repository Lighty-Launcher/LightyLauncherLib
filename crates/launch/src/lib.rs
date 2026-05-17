// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Minecraft launch orchestration.
//!
//! - [`launch`] coordinates the full launch pipeline (metadata → JRE →
//!   install → spawn) and exposes the [`Launch`](launch::Launch) trait.
//! - [`installer`] downloads and installs libraries, natives, the client
//!   JAR, mods, and assets in parallel.
//! - [`arguments`] turns a [`Version`](lighty_loaders::types::version_metadata::Version)
//!   plus runtime overrides into the final JVM/game argv.
//! - [`instance`] tracks running game processes and streams their console
//!   output back to the caller.

pub mod launch;
pub mod arguments;
pub mod errors;
pub mod installer;
pub mod instance;

pub use launch::{LaunchBuilder, LaunchConfig};
pub use installer::Installer;
pub use instance::{InstanceControl, InstanceError, InstanceResult};
