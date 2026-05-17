// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Serde mirrors of the `install_profile.json` shipped inside legacy
//! Forge installer JARs (Minecraft 1.4.x -> 1.12.2).

use std::collections::HashMap;

use serde::Deserialize;

use crate::loaders::vanilla::vanilla_metadata::Rule;

/// Top-level `install_profile.json` for legacy Forge installers.
#[derive(Debug, Deserialize, Clone)]
pub struct ForgeLegacyInstallProfile {
    pub install: ForgeLegacyInstallBlock,
    #[serde(rename = "versionInfo")]
    pub version_info: ForgeLegacyVersionInfo,
}

/// The `install` block — universal JAR shipped inside the installer ZIP.
#[derive(Debug, Deserialize, Clone)]
pub struct ForgeLegacyInstallBlock {
    #[serde(rename = "profileName")]
    pub profile_name: String,

    pub target: String,

    pub path: String,

    pub version: String,

    #[serde(rename = "filePath")]
    pub file_path: String,

    pub minecraft: String,
}

/// The `versionInfo` block — Mojang-style profile description.
#[derive(Debug, Deserialize, Clone)]
pub struct ForgeLegacyVersionInfo {
    pub id: String,

    #[serde(rename = "inheritsFrom", default)]
    pub inherits_from: Option<String>,

    #[serde(default)]
    pub assets: Option<String>,

    #[serde(rename = "mainClass")]
    pub main_class: String,

    #[serde(rename = "minecraftArguments")]
    pub minecraft_arguments: String,

    pub libraries: Vec<ForgeLegacyLibrary>,
}

/// One entry in `versionInfo.libraries`.
#[derive(Debug, Deserialize, Clone)]
pub struct ForgeLegacyLibrary {
    pub name: String,

    #[serde(default)]
    pub url: Option<String>,

    #[serde(default = "default_true")]
    pub clientreq: bool,

    #[serde(default = "default_true")]
    #[allow(dead_code)]
    pub serverreq: bool,

    #[serde(default)]
    pub rules: Option<Vec<Rule>>,

    #[serde(default)]
    pub natives: Option<HashMap<String, String>>,
}

fn default_true() -> bool {
    true
}
