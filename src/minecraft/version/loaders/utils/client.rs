use std::error::Error;
use std::path::Path;
use serde_json::Value;
use tokio::fs;
use crate::minecraft::version::loaders::utils::download::download_file;
use crate::minecraft::version::loaders::utils::manifest::Manifest;
use crate::minecraft::version::version::Version;

pub trait Client<'a> {
    async fn download_client(&self) -> Result<(), Box<dyn Error + Send + Sync>>;
}
impl<'a> Client<'a> for Version<'a> {
    async fn download_client(&self) -> Result<(), Box<dyn Error + Send + Sync>> {

        // Check if the client JAR already exists & set name of the jar
        let jar_path = self.get_game_dir().join(format!("{}.jar", self.name));

        if jar_path.exists() {
            println!("[LightyLauncher] Client JAR already exists, skipping download");
            return Ok(());
        }
        // Ensure parent directories exist
        if let Some(parent) = jar_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        let version_data = self.get_manifest_version().await?;

        if let Some(client) = version_data["downloads"]["client"].as_object() {
            let url = client["url"].as_str().ok_or("Client URL not found")?;
            let sha1 = client["sha1"].as_str().ok_or("Client SHA1 not found")?;
            let size = client["size"].as_u64().ok_or("Client size not found")?;

            println!("[LightyLauncher] Downloading client JAR from: {}", url);
            download_file(url, &jar_path, sha1, size).await?;

            println!("[LightyLauncher] Client JAR downloaded successfully");
        } else {
            return Err("Client download information not found in version data".into());
        }
        Ok(())
    }

}
