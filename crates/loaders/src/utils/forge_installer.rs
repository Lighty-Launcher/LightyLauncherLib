// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Serde mirrors of `install_profile.json` and `version.json` from modern
//! Forge/NeoForge installer JARs.

use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct ForgeInstallProfile {
    pub spec: i32,
    pub profile: String,
    pub version: String,
    pub icon: String,
    pub minecraft: String,
    pub json: String,
    pub logo: String,
    pub welcome: String,
    #[serde(rename = "mirrorList", default)]
    pub mirror_list: String,
    #[serde(rename = "hideExtract", default)]
    pub hide_extract: bool,
    pub data: HashMap<String, DataEntry>,
    pub processors: Vec<Processor>,
    pub libraries: Vec<ForgeLibrary>,
    #[serde(rename = "serverJarPath", default)]
    pub server_jar_path: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DataEntry {
    pub client: String,
    pub server: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Processor {
    #[serde(default)]
    pub sides: Vec<String>,
    pub jar: String,
    pub classpath: Vec<String>,
    pub args: Vec<String>,
    #[serde(default)]
    pub outputs: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ForgeLibrary {
    pub name: String,
    pub downloads: LibraryDownloads,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LibraryDownloads {
    pub artifact: Artifact,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Artifact {
    pub sha1: String,
    pub size: u64,
    pub url: String,
    pub path: String,
}

/// `version.json` — runtime classpath/main-class/arguments layout.
///
/// `arguments` (1.13+) and `minecraft_arguments` (back-ported 1.12.2)
/// are both optional.
#[derive(Debug, Deserialize, Clone)]
pub struct ForgeVersionManifest {
    #[serde(rename = "mainClass")]
    pub main_class: String,
    #[serde(default)]
    pub arguments: Option<ForgeArguments>,
    #[serde(rename = "minecraftArguments", default)]
    pub minecraft_arguments: Option<String>,
    pub libraries: Vec<ForgeLibrary>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ForgeArguments {
    pub game: Vec<String>,
    #[serde(default)]
    pub jvm: Vec<String>,
}
