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
