// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Per-vendor Java distribution providers.

mod temurin;
mod graalvm;
mod zulu;
mod liberica;

pub use temurin::build_temurin_url;
pub use graalvm::build_graalvm_url;
pub use zulu::build_zulu_url;
pub use liberica::build_liberica_url;
