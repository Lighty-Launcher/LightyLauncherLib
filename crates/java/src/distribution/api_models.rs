// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! API response wire-types for Java distribution providers.

use serde::Deserialize;

/// Zulu API response structure
#[derive(Debug, Deserialize)]
pub(super) struct ZuluPackage {
    pub download_url: String,
    pub name: String,
}

/// Foojay API response structure (for Liberica)
#[derive(Debug, Deserialize)]
pub(super) struct FoojayResponse {
    pub result: Vec<FoojayPackage>,
}

/// Individual package in Foojay response
#[derive(Debug, Deserialize)]
pub(super) struct FoojayPackage {
    pub links: FoojayLinks,
}

/// Links object in Foojay package
#[derive(Debug, Deserialize)]
pub(super) struct FoojayLinks {
    pub pkg_download_redirect: String,
}
