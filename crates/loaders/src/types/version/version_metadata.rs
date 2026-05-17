use std::collections::HashMap;
use serde::{Deserialize, Serialize};


/// Universal pivot type shared by every loader (Vanilla, Fabric, Forge, ...).
#[derive(Clone, Debug)]
pub enum VersionMetaData {
    JavaVersion(JavaVersion),
    MainClass(MainClass),
    Libraries(Vec<Library>),
    Mods(Vec<Mods>),
    Natives(Vec<Native>),
    AssetsIndex(AssetIndex),
    Assets(AssetsFile),
    Client(Client),
    Arguments(Arguments),
    Version(Version),
}

/// Full set of metadata required to install and launch a Minecraft version.
#[derive(Debug, Clone)]
pub struct Version {
    pub main_class: MainClass,
    pub java_version: JavaVersion,
    pub arguments: Arguments,
    pub libraries: Vec<Library>,
    pub mods: Option<Vec<Mods>>,
    pub natives: Option<Vec<Native>>,
    pub client: Option<Client>,
    pub assets_index: Option<AssetIndex>,
    pub assets: Option<AssetsFile>,
}

/// Game main class name (e.g. `net.minecraft.client.main.Main`).
#[derive(Debug, Clone)]
pub struct MainClass {
    pub main_class: String,
}

/// Required Java major version (e.g. `8`, `17`, `21`).
#[derive(Debug, Clone)]
pub struct JavaVersion {
    pub major_version: u8,
}

/// Game and JVM argument lists, post-placeholder substitution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Arguments {
    pub game: Vec<String>,
    pub jvm: Option<Vec<String>>,
}

/// Runtime classpath library entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    pub name: String,
    pub url: Option<String>,
    pub path: Option<String>,
    pub sha1: Option<String>,
    pub size: Option<u64>,
}

/// Mod JAR entry (used by `LightyUpdater` instances).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mods {
    pub name: String,
    pub url: Option<String>,
    pub path: Option<String>,
    pub sha1: Option<String>,
    pub size: Option<u64>,
}

/// Native library entry (per-OS shared object inside a Maven JAR).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Native {
    pub name: String,
    pub url: Option<String>,
    pub path: Option<String>,
    pub sha1: Option<String>,
    pub size: Option<u64>,
}

/// Game client JAR entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    pub name: String,
    pub url: Option<String>,
    pub path: Option<String>,
    pub sha1: Option<String>,
    pub size: Option<u64>,
}

/// Asset-index descriptor (used to fetch the actual asset list).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetIndex {
    pub id: String,
    pub url: String,
    pub sha1: String,
    pub size: u64,
    pub total_size: Option<u64>,
}

/// Materialized asset listing, keyed by virtual path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetsFile {
    pub objects: HashMap<String, Asset>,
}

/// Single content-addressed asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub hash: String,
    pub size: u64,
    pub url: Option<String>,
}
