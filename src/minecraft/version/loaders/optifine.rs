use std::error::Error;
use crate::minecraft::version::loaders::utils::assets::Assets;
use crate::minecraft::version::loaders::utils::librairies::Libraries;
use crate::minecraft::version::loaders::utils::natives::Natives;
use crate::minecraft::version::version::Version;
use crate::utils::hosts::HTTP_CLIENT;
use scraper::{Html, Selector};
use tokio::{fs as async_fs, fs};
use tokio::io::AsyncWriteExt;
use crate::mkdir;

use log::error;

pub trait OptifineLoader<'a> {
    async fn install_optifine(&self) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn download_optifine_client(&self) -> Result<(), Box<dyn Error + Send + Sync>>;
}


impl<'a> OptifineLoader<'a> for Version<'a> {
    async fn install_optifine(&self) -> Result<(), Box<dyn Error + Send + Sync>> {

        self.download_optifine_client().await?;
        self.download_libraries().await?;
        self.download_natives().await?;
        self.download_assets().await?;
        println!("[LightyLauncher] Installation complete for {}\n{:#?}", self.name, self.get_game_dir());
        Ok(())
    }
    async fn download_optifine_client(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        mkdir!(self.get_game_dir());
        // 1. Télécharger la page principale
        let downloads_page = HTTP_CLIENT.get("https://optifine.net/downloads")
            .send()
            .await?
            .text()
            .await?;
        let document = Html::parse_document(&downloads_page);

        // 2. Trouver les liens "Mirror"
        let mirror_selector = Selector::parse("a[href]").unwrap();
        let optifine_link = document
            .select(&mirror_selector)
            .filter_map(|element| {
                let href = element.value().attr("href")?;
                let text = element.text().collect::<Vec<_>>().join("");
                if href.contains("adloadx?f=OptiFine_")
                    && text.contains("Mirror")
                    && href.contains(&self.minecraft_version)
                {
                    Some(href.to_string())
                } else {
                    None
                }
            })
            .next()
            .ok_or_else(|| format!("No OptiFine Mirror found for Minecraft {}", self.minecraft_version))?;

        let adloadx_url = if optifine_link.starts_with("http") {
            optifine_link
        } else {
            format!("https://optifine.net/{}", optifine_link)
        };
        println!("[LightyLauncher] Found OptiFine Mirror: {}", adloadx_url);

        // 3. Charger la page adloadx
        let page = HTTP_CLIENT.get(&adloadx_url)
            .send()
            .await?
            .text()
            .await?;
        let document = Html::parse_document(&page);

        // 4. Trouver le lien final de téléchargement
        let download_selector = Selector::parse("a[href]").unwrap();
        let download_x = document
            .select(&download_selector)
            .filter_map(|element| element.value().attr("href"))
            .find(|href| href.contains("downloadx?f="))
            .ok_or("Could not find final download link on OptiFine page")?;

        let x_value = download_x
            .split("&x=")
            .nth(1)
            .ok_or("Could not extract x parameter from download link")?;

        // 5. Récupérer le vrai nom du fichier JAR
        let jar_name = download_x
            .split("f=")
            .nth(1)
            .and_then(|part| part.split("&").next())
            .ok_or("Failed to parse jar name")?;

        let download_url = format!("https://optifine.net/downloadx?f={}&x={}", jar_name, x_value);
        println!("[LightyLauncher] Downloading OptiFine from: {}", download_url);

        // 6. Télécharger et sauvegarder
        let response = HTTP_CLIENT.get(&download_url).send().await?.bytes().await?;
        let output_path = self.get_game_dir().join(format!("{}.jar",self.name));

        let mut file = async_fs::File::create(&output_path).await?;
        file.write_all(&response).await?;

        println!(
            "[LightyLauncher] OptiFine {} downloaded to {}",
            jar_name,
            output_path.display()
        );

        Ok(())
    }

}