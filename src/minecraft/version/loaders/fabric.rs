use std::error::Error;
use std::fs;
use serde::Deserialize;
use tokio::io::AsyncWriteExt;
use tokio::try_join;
use crate::minecraft::version::loaders::utils::assets::Assets;
use crate::minecraft::version::loaders::utils::client::Client;
use crate::minecraft::version::loaders::utils::librairies::Libraries;
use crate::minecraft::version::loaders::utils::natives::Natives;
use crate::minecraft::version::version::Version;
use crate::utils::hosts::HTTP_CLIENT;


#[derive(Debug, Deserialize)]
struct FabricLibrary {
    name: String,
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FabricProfile {
    libraries: Vec<FabricLibrary>,
}

pub trait FabricLoader<'a> {
    async fn get_fabric_manifest(&self) -> Result<String, Box<dyn Error + Send + Sync>>;
    async fn install_fabric(&self) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn download_fabric_libraries(&self) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn get_latest_fabric_loader_version(&self) -> Result<String, Box<dyn Error + Send + Sync>>;
}
impl<'a> FabricLoader<'a> for Version<'a> {
    async fn get_fabric_manifest(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        let url = format!(
            "https://meta.fabricmc.net/v2/versions/loader/{}/{}/profile/json",
            self.minecraft_version, self.loader_version
        );
        // Télécharger et parser le JSON
        let response = HTTP_CLIENT.get(&url).send().await?;
        let version_data: serde_json::Value = response.json().await?;

        // Extraire la chaîne correctement
        let main_class = version_data["mainClass"]
            .as_str()
            .ok_or("Main class not found in Fabric manifest")?;

        println!("[FabricLoader] Fabric Manifest: {}", main_class);
        Ok(main_class.to_string())
    }
    async fn install_fabric(&self) -> Result<(), Box<dyn Error + Send + Sync>> {

        try_join!(
            self.download_client(),
            self.download_libraries(),
            self.download_fabric_libraries(),
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
    
    

    async fn download_fabric_libraries(&self) -> Result<(), Box<dyn Error + Send + Sync>> {


        // 1. Télécharger le profil JSON
        let url = format!(
            "https://meta.fabricmc.net/v2/versions/loader/{}/{}/profile/json",
            self.minecraft_version, self.loader_version
        );
        println!("Downloading Fabric profile from: {}", url);
        let resp = HTTP_CLIENT.get(&url).send().await?;
        let profile: FabricProfile = resp.json().await?;

        // 2. Télécharger toutes les libraries
        for lib in profile.libraries {
            let parts: Vec<&str> = lib.name.split(':').collect();
            if parts.len() != 3 {
                continue; // skip mal formé
            }

            let group = parts[0].replace('.', "/");
            let artifact = parts[1];
            let version = parts[2];

            let file_name = format!("{}-{}.jar", artifact, version);
            let path = format!("{}/{}/{}/{}", group, artifact, version, file_name);

            // Source Maven officielle ou spécifique
            let base_url = lib.url.unwrap_or_else(|| "https://maven.fabricmc.net/".to_string());
            let full_url = format!("{}{}", base_url, path);

            // Destination locale
            let local_path = self.get_libraries_dir().join(&path);
            if !local_path.exists() {
                // Créer dossier parent
                if let Some(parent) = local_path.parent() {
                    fs::create_dir_all(parent)?;
                }

                println!("Downloading {} -> {:?}", lib.name, local_path);
                let mut resp = HTTP_CLIENT.get(&full_url).send().await?;
                let mut file = tokio::fs::File::create(&local_path).await?;
                println!("Full URL: {}", full_url);
                println!("Local path: {:?}", local_path);
                while let Some(chunk) = resp.chunk().await? {
                    file.write_all(&chunk).await?;
                }
            } else {
                println!("Already exists: {:?}", local_path);
            }
        }

        Ok(())
    }
    async fn get_latest_fabric_loader_version(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        let url = "https://meta.fabricmc.net/v2/versions/loader";

        // Télécharger et parser le JSON
        let response = HTTP_CLIENT.get(url).send().await?;

        // Vérifier si la requête a réussi
        if !response.status().is_success() {
            return Err(format!("Échec de la requête API Fabric: {}", response.status()).into());
        }

        // Analyser la réponse JSON
        let versions: Vec<serde_json::Value> = response.json().await?;

        // Vérifier si nous avons des versions disponibles
        if versions.is_empty() {
            return Err("Aucune version du loader Fabric n'a été trouvée".into());
        }

        // La première version dans la liste est la plus récente
        let latest_version = versions[0]["version"]
            .as_str()
            .ok_or("Format de version invalide dans la réponse de l'API Fabric")?;

        println!("[FabricLoader] Dernière version disponible: {}", latest_version);

        Ok(latest_version.to_string())
    }


}