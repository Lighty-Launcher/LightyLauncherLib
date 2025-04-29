use std::error::Error;
use crate::minecraft::version::loaders::utils::assets::Assets;
use crate::minecraft::version::loaders::utils::client::Client;
use crate::minecraft::version::loaders::utils::librairies::Libraries;
use crate::minecraft::version::loaders::utils::natives::Natives;
use crate::minecraft::version::version::Version;

pub trait VanillaLoader<'a> {
    async fn install_vanilla(&self) -> Result<(), Box<dyn Error + Send + Sync>>;
}

use tokio::try_join;

impl<'a> VanillaLoader<'a> for Version<'a> {
    async fn install_vanilla(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        try_join!(
            self.download_client(),
            self.download_libraries(),
            self.download_natives(),
            self.download_assets(),
        )?;

        println!(
            "[LightyLauncher] Installation complete for {}\n{:#?}",
            self.name,
            self.get_game_dir()
        );
        Ok(())
    }
}
