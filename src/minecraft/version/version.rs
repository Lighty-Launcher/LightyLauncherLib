use std::error::Error;
use std::path::{PathBuf};
use directories::ProjectDirs;
use once_cell::sync::Lazy;
use version_compare;
use crate::minecraft::version::loaders::fabric::FabricLoader;
use crate::minecraft::version::loaders::neoforge::NeoForgeLoader;
use crate::minecraft::version::loaders::optifine::OptifineLoader;
use crate::minecraft::version::loaders::quilt::QuiltLoader;
use crate::minecraft::version::loaders::vanilla::VanillaLoader;

#[derive(Debug)]
pub(crate) struct Version<'a> {
    pub(crate) name :String,
    pub(crate) loader:String,
    pub(crate) loader_version:String,
    pub(crate) minecraft_version: String,
    project_dirs: &'a Lazy<ProjectDirs>,
}

impl<'a> Version<'a> {

    pub fn new(name: &str, loader: &str, loader_version: &str, minecraft_version: &str, project_dirs: &'a Lazy<ProjectDirs>) -> Self
    {
        Self { name: name.to_string(), loader: loader.to_string(), loader_version: loader_version.to_string(), minecraft_version: minecraft_version.to_string(), project_dirs, }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn get_game_dir(&self) -> PathBuf {
        self.project_dirs.data_dir().join(&self.name)
    }
    pub fn get_libraries_dir(&self) -> PathBuf {
        self.get_game_dir().join("libraries")
    }
    pub fn get_assets_dir(&self) -> PathBuf {
        self.get_game_dir().join("assets")
    }
    pub fn get_natives_dir(&self) -> PathBuf {
        self.get_game_dir().join("natives")
    }
    pub async fn uninstall_version(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        println!("[LightyLauncher] Uninstalling: {}", self.name);
        // Remove the game directory
        if self.get_game_dir().exists() {
            tokio::fs::remove_dir_all(&self.get_game_dir()).await?;
        }
        println!("[LightyLauncher] Uninstallation complete for {}", self.name);
        Ok(())
    }

    // check the type of the loader and install with the correct version
    pub async fn install_version(&self) -> Result<(), Box<dyn Error + Send + Sync>> {


        match self.loader.as_ref() {
            "fabric" => {
                self.install_fabric().await?;
            }
            "neoforge" => {
                self.install_neoforge().await?;
            }
            "optifine" => {
                self.install_optifine().await?;
            }
            "quilt" => {
                self.install_quilt().await?;
            }
            "vanilla" => {
                self.install_vanilla().await?;

            }
            "forge" => {
                // TODO
            }
            _ => {
                println!(
                    "[LightyLauncher] the loader: '{}' is not a valid loader. Please check ma-documentation.fr",
                    &self.loader
                );
            }
        }
        println!("[LightyLauncher] Installation complete for {} ", self.name);
        println!("[LightyLauncher] the Game Directory is '{:#?}'", self.get_game_dir());
        Ok(())
    }




}







