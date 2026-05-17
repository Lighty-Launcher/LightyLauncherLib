// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Serde wire-types for `manifest.json` inside a CurseForge modpack `.zip`.

use lighty_core::QueryError;
use lighty_loaders::types::Loader;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CfModpackManifest {
    pub minecraft: CfMinecraftSpec,
    #[serde(rename = "manifestType")]
    pub manifest_type: String,
    #[serde(rename = "manifestVersion")]
    pub manifest_version: u32,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub author: Option<String>,
    pub files: Vec<CfModpackFile>,
    pub overrides: String,
}

#[derive(Debug, Deserialize)]
pub struct CfMinecraftSpec {
    pub version: String,
    #[serde(rename = "modLoaders")]
    pub mod_loaders: Vec<CfModLoader>,
}

#[derive(Debug, Deserialize)]
pub struct CfModLoader {
    pub id: String,
    pub primary: bool,
}

#[derive(Debug, Deserialize)]
pub struct CfModpackFile {
    #[serde(rename = "projectID")]
    pub project_id: u32,
    #[serde(rename = "fileID")]
    pub file_id: u32,
    #[serde(default = "default_true")]
    pub required: bool,
}

fn default_true() -> bool {
    true
}

impl CfModpackManifest {
    /// Returns the primary mod loader entry, or the first one if none is
    /// marked primary (older packs sometimes omit the flag).
    pub fn primary_loader(&self) -> Option<&CfModLoader> {
        self.minecraft
            .mod_loaders
            .iter()
            .find(|l| l.primary)
            .or_else(|| self.minecraft.mod_loaders.first())
    }
}

impl CfModLoader {
    /// Parses the `id` field into `(Loader, version_string)`.
    ///
    /// Examples: `"fabric-0.16.9"` ⇒ `(Loader::Fabric, "0.16.9")`.
    pub fn parse(&self) -> Result<(Loader, String), QueryError> {
        let (loader_str, version) =
            self.id.split_once('-').ok_or_else(|| QueryError::Conversion {
                message: format!("Invalid CurseForge modLoader id: {}", self.id),
            })?;
        let loader = match loader_str {
            "fabric" => Loader::Fabric,
            "forge" => Loader::Forge,
            "neoforge" => Loader::NeoForge,
            "quilt" => Loader::Quilt,
            other => {
                return Err(QueryError::UnsupportedLoader(format!(
                    "CurseForge modpack uses loader '{other}', not supported"
                )))
            }
        };
        Ok((loader, version.to_string()))
    }
}
