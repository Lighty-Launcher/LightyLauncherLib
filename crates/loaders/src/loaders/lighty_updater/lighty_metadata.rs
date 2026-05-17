//! Serde types describing a LightyUpdater server response.

use serde::{Deserialize, Serialize};

/// Server response listing every instance the LightyUpdater publishes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServersResponse {
    servers: Vec<ServerInfo>,
}

impl ServersResponse {
    /// Finds the server entry matching `name`, if any.
    pub fn find_by_name(&self, name: &str) -> Option<&ServerInfo> {
        self.servers.iter().find(|s| s.name == name)
    }
}

/// Per-server info entry returned by the LightyUpdater listing endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    name: String,
    loader: String,
    loader_version: String,
    minecraft_version: String,
    url: String,
    last_update: String,
}

impl ServerInfo {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn loader(&self) -> &str {
        &self.loader
    }

    pub fn loader_version(&self) -> &str {
        &self.loader_version
    }

    pub fn minecraft_version(&self) -> &str {
        &self.minecraft_version
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn last_update(&self) -> &str {
        &self.last_update
    }
}

/// Metadata document returned by a LightyUpdater server. Every field is
/// optional — server supplies only the overrides, base loader fills the rest.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LightyMetadata {
    #[serde(skip)]
    pub server_info: Option<ServerInfo>,
    pub main_class: Option<MainClass>,
    pub java_version: Option<JavaVersion>,
    pub arguments: Option<Arguments>,
    pub libraries: Option<Vec<Library>>,
    pub natives: Option<Vec<Native>>,
    pub client: Option<Client>,
    pub assets: Option<Vec<Asset>>,
    pub mods: Option<Vec<Mod>>,
}

impl Default for LightyMetadata {
    fn default() -> Self {
        Self {
            server_info: None,
            main_class: None,
            java_version: None,
            arguments: None,
            libraries: None,
            natives: None,
            client: None,
            assets: None,
            mods: None,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MainClass {
    pub main_class: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaVersion {
    pub major_version: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Arguments {
    pub game: Vec<String>,
    pub jvm: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mod {
    pub name: String,
    pub url: String,
    pub path: String,
    pub sha1: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Native {
    pub name: String,
    pub url: String,
    pub path: String,
    pub sha1: String,
    pub size: u64,
    pub os: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    pub name: String,
    pub url: String,
    pub path: String,
    pub sha1: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub hash: String,
    pub size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}
