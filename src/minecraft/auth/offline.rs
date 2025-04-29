struct OfflineAuthenticator {
    username: String,
}
impl OfflineAuthenticator {
    pub fn new(username: String) -> Self {
        Self { username }
    }

    pub fn authenticate(&self) -> Result<OfflineProfile> {
        let profile = OfflineProfile::new(self.username.clone());
        Ok(profile)
    }
}