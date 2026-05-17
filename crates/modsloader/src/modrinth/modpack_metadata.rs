// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Serde wire-types for `modrinth.index.json` inside a `.mrpack` archive.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MrpackManifest {
    #[serde(rename = "formatVersion")]
    pub format_version: u32,
    pub game: String,
    #[serde(rename = "versionId")]
    pub version_id: String,
    pub name: String,
    #[serde(default)]
    pub summary: Option<String>,
    pub files: Vec<MrpackFile>,
    pub dependencies: MrpackDependencies,
}

#[derive(Debug, Deserialize)]
pub struct MrpackFile {
    pub path: String,
    pub hashes: MrpackHashes,
    #[serde(default)]
    pub env: Option<MrpackEnv>,
    pub downloads: Vec<String>,
    #[serde(rename = "fileSize")]
    pub file_size: u64,
}

#[derive(Debug, Deserialize)]
pub struct MrpackHashes {
    pub sha1: String,
    #[serde(default)]
    pub sha512: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MrpackEnv {
    pub client: String,
    pub server: String,
}

#[derive(Debug, Deserialize)]
pub struct MrpackDependencies {
    pub minecraft: String,
    #[serde(rename = "fabric-loader", default)]
    pub fabric_loader: Option<String>,
    #[serde(default)]
    pub forge: Option<String>,
    #[serde(default)]
    pub neoforge: Option<String>,
    #[serde(rename = "quilt-loader", default)]
    pub quilt_loader: Option<String>,
}

impl MrpackFile {
    /// Whether this file should be installed for a client-side launch.
    /// Returns `true` when no `env` is set (Modrinth's older packs).
    pub fn is_client_required(&self) -> bool {
        match &self.env {
            None => true,
            Some(e) => e.client == "required",
        }
    }
}
