// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Serde mirrors of the CurseForge Core-API JSON payloads.

use serde::Deserialize;

pub const MOD_LOADER_FORGE: u8 = 1;
pub const MOD_LOADER_FABRIC: u8 = 4;
pub const MOD_LOADER_QUILT: u8 = 5;
pub const MOD_LOADER_NEOFORGE: u8 = 6;

pub const CLASS_MOD: u32 = 6;
pub const CLASS_RESOURCE_PACK: u32 = 12;
pub const CLASS_SHADER_PACK: u32 = 6552;
#[allow(dead_code)]
pub const CLASS_MODPACK: u32 = 4471;

pub const HASH_ALGO_SHA1: u8 = 1;
#[allow(dead_code)]
pub const HASH_ALGO_MD5: u8 = 2;

#[allow(dead_code)]
pub const DEP_EMBEDDED_LIBRARY: u8 = 1;
#[allow(dead_code)]
pub const DEP_OPTIONAL: u8 = 2;
pub const DEP_REQUIRED: u8 = 3;
#[allow(dead_code)]
pub const DEP_TOOL: u8 = 4;
#[allow(dead_code)]
pub const DEP_INCOMPATIBLE: u8 = 5;
#[allow(dead_code)]
pub const DEP_INCLUDE: u8 = 6;

#[derive(Debug, Deserialize)]
pub struct CurseForgeEnvelope<T> {
    pub data: T,
}

#[derive(Debug, Deserialize)]
pub struct CurseForgeMod {
    #[serde(rename = "classId")]
    pub class_id: u32,
}

#[derive(Debug, Deserialize)]
pub struct CurseForgeFile {
    pub id: u32,
    #[serde(rename = "modId")]
    pub mod_id: u32,
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "downloadUrl", default)]
    pub download_url: Option<String>,
    #[serde(rename = "fileLength", default)]
    pub file_length: u64,
    #[serde(default)]
    pub hashes: Vec<CurseForgeHash>,
    #[serde(default)]
    pub dependencies: Vec<CurseForgeDependency>,
}

#[derive(Debug, Deserialize)]
pub struct CurseForgeHash {
    pub value: String,
    pub algo: u8,
}

#[derive(Debug, Deserialize)]
pub struct CurseForgeDependency {
    #[serde(rename = "modId")]
    pub mod_id: u32,
    #[serde(rename = "relationType")]
    pub relation_type: u8,
}
