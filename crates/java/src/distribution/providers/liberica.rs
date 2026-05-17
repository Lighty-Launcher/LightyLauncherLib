// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! BellSoft Liberica distribution provider (Foojay API).

use crate::errors::{DistributionError, DistributionResult};
use crate::distribution::api_models::FoojayResponse;
use lighty_core::system::{ARCHITECTURE, OS};
use lighty_core::hosts::HTTP_CLIENT;

/// Builds BellSoft Liberica download URL via the Foojay API.
pub async fn build_liberica_url(version: &u8) -> DistributionResult<String> {
    let os_name = OS.get_zulu_name()?;
    let arch = ARCHITECTURE.get_simple_name()?;
    let ext = OS.get_archive_type()?;

    let api_url = format!(
        "https://api.foojay.io/disco/v3.0/packages?distro=liberica&version={}&operating_system={}&architecture={}&archive_type={}&package_type=jre&release_status=ga&latest=available",
        version, os_name, arch, ext
    );

    let response = HTTP_CLIENT
        .get(&api_url)
        .header("User-Agent", "Lighty-Launcher-Rust")
        .send()
        .await
        .map_err(|e| DistributionError::ApiError {
            distribution: "Liberica",
            error: e.to_string(),
        })?;

    let foojay_response: FoojayResponse = response
        .json()
        .await
        .map_err(|e| DistributionError::JsonParseError {
            distribution: "Liberica",
            error: e.to_string(),
        })?;

    foojay_response.result
        .into_iter()
        .next()
        .map(|pkg| pkg.links.pkg_download_redirect)
        .ok_or(DistributionError::NoPackagesFound {
            distribution: "Liberica",
        })
}
