// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Serde mirrors of the Modrinth Labrinth-API JSON payloads.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ModrinthProject {
    pub project_type: String,
}

#[derive(Debug, Deserialize)]
pub struct ModrinthVersion {
    pub id: String,
    pub version_number: String,
    pub files: Vec<ModrinthFile>,
    #[serde(default)]
    pub dependencies: Vec<ModrinthDependency>,
}

#[derive(Debug, Deserialize)]
pub struct ModrinthFile {
    pub url: String,
    pub filename: String,
    pub size: u64,
    pub hashes: ModrinthHashes,
    #[serde(default)]
    pub primary: bool,
}

#[derive(Debug, Deserialize)]
pub struct ModrinthHashes {
    pub sha1: String,
}

#[derive(Debug, Deserialize)]
pub struct ModrinthDependency {
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub version_id: Option<String>,
    pub dependency_type: String,
}
