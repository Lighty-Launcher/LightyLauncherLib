/// The supported Minecraft mod loaders.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Loader {
    Fabric,
    NeoForge,
    Optifine,
    Quilt,
    Vanilla,
    Forge,
    LightyUpdater,
}

impl Loader {
    /// The name a LightyUpdater server puts in its manifest. An unknown
    /// name is an error rather than a silent fallback: a typo would
    /// otherwise install the wrong loader and fail much later.
    pub fn from_server_name(name: &str) -> Result<Self, lighty_core::QueryError> {
        match name {
            "vanilla" => Ok(Loader::Vanilla),
            "fabric" => Ok(Loader::Fabric),
            "quilt" => Ok(Loader::Quilt),
            "forge" => Ok(Loader::Forge),
            "neoforge" => Ok(Loader::NeoForge),
            _ => Err(lighty_core::QueryError::UnknownLoader {
                name: name.to_string(),
            }),
        }
    }
}
