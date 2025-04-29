pub(crate) struct ForgeLoader {
    version: String,
    loader_version: String,
}
impl ForgeLoader {
    pub fn new(version: String, loader_version: String) -> Self {
        Self {
            version,
            loader_version,
        }
    }
    pub fn get_version(&self) -> &str {
        &self.version
    }

    pub fn get_loader_version(&self) -> &str {
        &self.loader_version
    }
}