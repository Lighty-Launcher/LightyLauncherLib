//! Serde mirrors of the JSON documents returned by Mojang's `piston-meta`.

use std::collections::HashMap;
use serde::Deserialize;

/// Top-level `version_manifest_v2.json` document.
#[derive(Debug, Deserialize)]
pub struct PistonMetaManifest {
    pub latest: Latest,
    pub versions: Vec<VersionInfo>,
}

#[derive(Debug, Deserialize)]
pub struct Latest {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Deserialize)]
pub struct VersionInfo {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: String,
    pub url: String,
    pub time: String,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
    pub sha1: String,
    #[serde(rename = "complianceLevel")]
    pub compliance_level: i32,
}


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VanillaMetaData {
    pub id: String,
    pub main_class: String,
    #[serde(rename="type")]
    pub type_field: String,
    pub release_time: String,
    pub time: String,
    pub minimum_launcher_version: Option<u32>,

    pub asset_index: AssetIndex,
    pub assets: String,
    pub compliance_level: Option<u32>,
    pub java_version: Option<JavaVersion>,

    pub downloads: Downloads,
    pub libraries: Vec<Library>,

    #[serde(default)]
    pub minecraft_arguments: Option<String>,

    #[serde(default)]
    pub arguments: Option<Arguments>,

    #[serde(default)]
    pub logging: Option<Logging>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    pub total_size: Option<u64>,
    pub url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaVersion {
    pub component: String,
    pub major_version: u32,
}

#[derive(Debug, Deserialize)]
pub struct Downloads {
    pub client: Option<DownloadEntry>,
    pub server: Option<DownloadEntry>,
    #[serde(rename = "windows_server")]
    pub windows_server: Option<DownloadEntry>,
    #[serde(rename = "client_mappings")]
    pub client_mappings: Option<DownloadEntry>,
    #[serde(rename = "server_mappings")]
    pub server_mappings: Option<DownloadEntry>,
}

#[derive(Debug, Deserialize)]
pub struct DownloadEntry {
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Library {
    pub name: String,
    pub downloads: LibraryDownloads,
    #[serde(default)]
    pub rules: Option<Vec<Rule>>,
    #[serde(default)]
    pub natives: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct LibraryDownloads {
    pub artifact: Option<Artifact>,
    #[serde(default)]
    pub classifiers: Option<std::collections::HashMap<String, Artifact>>,
}

#[derive(Debug, Deserialize)]
pub struct Artifact {
    pub path: String,
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Rule {
    pub action: String,
    #[serde(default)]
    pub os: Option<RuleOS>,
    #[serde(default)]
    pub features: Option<std::collections::HashMap<String, bool>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RuleOS {
    pub name: Option<String>,
    pub arch: Option<String>,
}

#[derive(Debug, Deserialize)]
#[derive(Clone)]
#[derive(Default)]
pub struct Arguments {
    // Each entry is either a String or an Object.
    #[serde(default)]
    pub game: Vec<serde_json::Value>,
    #[serde(default)]
    pub jvm: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct Logging {
    pub client: Option<LoggingEntry>,
}

#[derive(Debug, Deserialize)]
pub struct LoggingEntry {
    pub argument: String,
    pub file: DownloadEntryWithId,
    pub r#type: String,
}

#[derive(Debug, Deserialize)]
pub struct DownloadEntryWithId {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    pub url: String,
}



#[derive(Debug, Deserialize)]
pub struct VanillaAssetFile {
    pub objects: HashMap<String, Asset>,
}

#[derive(Debug, Deserialize)]
pub struct Asset {
    pub hash: String,
    pub size: u64,
}