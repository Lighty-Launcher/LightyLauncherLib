// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Azul Zulu distribution provider.

use crate::errors::{DistributionError, DistributionResult};
use crate::distribution::api_models::ZuluPackage;
use lighty_core::system::{ARCHITECTURE, OS};
use lighty_core::hosts::HTTP_CLIENT;

/// Builds Zulu download URL via the Azul API.
pub async fn build_zulu_url(version: &u8) -> DistributionResult<String> {
    use lighty_core::system::{Architecture, OperatingSystem};

    let os_name = OS.get_zulu_name()?;
    let ext = OS.get_zulu_ext()?;

    // Java 8 on macOS ARM64: pre-1.19 Minecraft only ships x86_64 natives, so use
    // x64 Java to run everything under Rosetta 2.
    let arch_name = if *version == 8 && OS == OperatingSystem::OSX && ARCHITECTURE == Architecture::AARCH64 {
        "x64"
    } else {
        ARCHITECTURE.get_zulu_arch()?
    };

    let api_url = format!(
        "https://api.azul.com/metadata/v1/zulu/packages?os={}&arch={}&archive_type={}&java_package_type=jre&release_status=ga&java_version={}&latest=true",
        os_name, arch_name, ext, version
    );

    let response = HTTP_CLIENT
        .get(&api_url)
        .header("User-Agent", "Lighty-Launcher-Rust")
        .send()
        .await
        .map_err(|e| DistributionError::ApiError {
            distribution: "Zulu",
            error: e.to_string(),
        })?;

    let packages: Vec<ZuluPackage> = response
        .json()
        .await
        .map_err(|e| DistributionError::JsonParseError {
            distribution: "Zulu",
            error: e.to_string(),
        })?;

    // Skip JavaFX-bundled packages (-fx-).
    packages
        .into_iter()
        .find(|pkg| !pkg.name.contains("-fx-"))
        .map(|pkg| pkg.download_url)
        .ok_or(DistributionError::NoPackagesFound {
            distribution: "Zulu",
        })
}
