use std::error::Error;
use serde_json::Value;
use crate::minecraft::version::loaders::fabric::FabricLoader;
use crate::minecraft::version::loaders::neoforge::NeoForgeLoader;
use crate::minecraft::version::loaders::quilt::QuiltLoader;
use crate::minecraft::version::version::Version;

pub trait Manifest<'a> {

    async fn get_manifest_version(&self) -> Result<Value, Box<dyn Error + Send + Sync>>;
    async fn get_java_from_manifest(&self) -> Result<u32, Box<dyn Error + Send + Sync>>;
    async fn get_main_class_from_manifest(&self) -> Result<String, Box<dyn Error + Send + Sync>>;

}
impl<'a> Manifest<'a> for Version<'a> {
    async fn get_manifest_version(&self) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let manifest_url = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
        println!("[LightyLauncher] Fetching manifest from: {}", manifest_url);

        // Retrieve global manifest
        let manifest: Value = reqwest::get(manifest_url).await?.json().await?;

        // Find desired version
        let version_info = manifest["versions"]
            .as_array()
            .and_then(|versions| {
                versions.iter().find(|v| {
                    v["id"].as_str().map_or(false, |id| id == &self.minecraft_version)
                })
            })
            .ok_or(format!("Version {} not found in manifest", &self.minecraft_version))?;

        let version_url = version_info["url"]
            .as_str()
            .ok_or("URL field missing in version info")?;

        println!("[LightyLauncher] Fetching version JSON from: {}", version_url);

        // Retrieve complete JSON for target version
        let version_data: Value = reqwest::get(version_url).await?.json().await?;

        Ok(version_data)
    }
    async fn get_java_from_manifest(&self) -> Result<u32, Box<dyn Error + Send + Sync>> {
        let version_data = self.get_manifest_version().await?;
        let java_version = version_data["javaVersion"]["majorVersion"]
            .as_u64()
            .ok_or("Java majorVersion not found in manifest")?;

        Ok(java_version as u32)
    }

    async fn get_main_class_from_manifest(&self) -> Result<String, Box<dyn Error + Send + Sync>> {

        match self.loader.as_str() {
            "fabric" => {
                println!("[LightyLauncher] Fabric Main class {:#?}", self.get_fabric_manifest().await?);
                Ok(self.get_fabric_manifest().await?)
            }
            "vanilla" => {
                let version_data = self.get_manifest_version().await?;
                let main_class = version_data["mainClass"]
                    .as_str()
                    .ok_or("Main class not found in manifest")?;
                Ok(main_class.to_string())
            }
            "quilt" => {
                println!("[LightyLauncher] Quilt Main class {:#?}", self.get_quilt_manifest().await?);
                Ok(self.get_quilt_manifest().await?)
            }
            "optifine" => {
                //TODO: OptiFine doesn't have a standard manifest, so we need to handle it differently
                // let version_data = self.get_manifest_version().await?;
                // let main_class = version_data["mainClass"]
                //     .as_str()
                //     .ok_or("Main class not found in manifest")?;
                // Ok(main_class.to_string())
                Ok("optifine.InstallerFrame".to_string())
            }
            "neoforge" => {
                Ok(self.get_neoforge_manifest().await?)
            }

            _ => {
                return Err("Loader not supported".into());
            }

        }

    }




}
