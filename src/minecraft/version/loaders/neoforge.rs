use std::error::Error;
use std::fs;
use serde::Deserialize;
use tokio::io::AsyncWriteExt;
use crate::minecraft::version::loaders::utils::assets::Assets;
use crate::minecraft::version::loaders::utils::client::Client;
use crate::minecraft::version::loaders::utils::librairies::Libraries;
use crate::minecraft::version::loaders::utils::natives::Natives;
use crate::minecraft::version::version::Version;
use crate::utils::hosts::HTTP_CLIENT;
use std::process::Command;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct NeoForgeLibrary {
    name: String,
    url: Option<String>,
    downloads: Option<LibraryDownloads>,
}

#[derive(Debug, Deserialize)]
struct LibraryDownloads {
    artifact: Option<ArtifactInfo>,
}

#[derive(Debug, Deserialize)]
struct ArtifactInfo {
    path: String,
    url: String,
    sha1: Option<String>,
    size: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct NeoForgeProfile {
    libraries: Vec<NeoForgeLibrary>,
    mainClass: String,
}

pub trait NeoForgeLoader<'a> {
    async fn get_neoforge_manifest(&self) -> Result<String, Box<dyn Error + Send + Sync>>;
    async fn install_neoforge(&self) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn download_neoforge_libraries(&self) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn get_latest_neoforge_version(&self) -> Result<String, Box<dyn Error + Send + Sync>>;
    fn is_old_neoforge(&self) -> bool;
    fn get_version_id(&self) -> String;
}

impl<'a> NeoForgeLoader<'a> for Version<'a> {
    async fn get_neoforge_manifest(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        let version_id = self.get_version_id();
        let json_path = self.get_game_dir().join(format!("{}.json", version_id));

        // Si le fichier n'existe pas, nous ne pouvons pas obtenir le manifeste
        if !json_path.exists() {
            return Err("NeoForge manifest JSON not found".into());
        }

        // Lire et parser le fichier JSON
        let json_content = fs::read_to_string(json_path)?;
        let version_data: serde_json::Value = serde_json::from_str(&json_content)?;

        // Extraire la classe principale
        let main_class = version_data["mainClass"]
            .as_str()
            .ok_or("Main class not found in NeoForge manifest")?;

        println!("[NeoForgeLoader] NeoForge Manifest: {}", main_class);
        Ok(main_class.to_string())
    }

    async fn install_neoforge(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        // Télécharger d'abord le client Minecraft
        self.download_client().await?;
        // Télécharger les bibliothèques standard et NeoForge
        self.download_libraries().await?;
        self.download_natives().await?;
        self.download_assets().await?;
        // Installer NeoForge via l'installateur
        self.download_and_run_installer().await?;
        self.download_neoforge_libraries().await?;

        println!("Installing NeoForge libraries completed");
        // Télécharger les composants restants



        println!("[LightyLauncher] Installation complete for {}\n{:#?}", self.name, self.get_game_dir());
        Ok(())
    }

    async fn download_neoforge_libraries(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let version_id = self.get_version_id();
        let game_dir = self.get_game_dir();
        let json_path = game_dir.join(format!("{}.json", version_id));

        // Si le dossier du jeu n'existe pas, le créer
        if !game_dir.exists() {
            fs::create_dir_all(&game_dir)?;
        }

        // Télécharger l'installer NeoForge
        let installer_url = if self.is_old_neoforge() {
            let path_version = format!("{}-{}", self.minecraft_version, self.loader_version);
            let file_prefix = format!("forge-{}", self.minecraft_version);
            format!(
                "https://maven.neoforged.net/releases/net/neoforged/forge/{}/{}-{}-installer.jar",
                path_version, file_prefix, self.loader_version
            )
        } else {
            format!(
                "https://maven.neoforged.net/releases/net/neoforged/neoforge/{}/{}-installer.jar",
                self.loader_version, version_id
            )
        };
        let installer_path = game_dir.join(format!("{}-installer.jar", version_id));

        if !installer_path.exists() {
            println!("Downloading NeoForge installer from: {}", installer_url);
            let mut response = HTTP_CLIENT.get(&installer_url).send().await?;
            let mut file = tokio::fs::File::create(&installer_path).await?;

            while let Some(chunk) = response.chunk().await? {
                file.write_all(&chunk).await?;
            }
        } else {
            println!("Installer already exists: {:?}", installer_path);
        }

        // Vérifier si le fichier JSON existe
        if !json_path.exists() {
            return Err(format!("NeoForge JSON file not found: {:?}", json_path).into());
        }

        // Lire et parser le JSON
        let json_content = fs::read_to_string(&json_path)?;
        let profile: NeoForgeProfile = serde_json::from_str(&json_content)
            .map_err(|e| format!("Failed to parse NeoForge profile JSON: {}", e))?;

        // Télécharger toutes les bibliothèques
        for lib in profile.libraries {
            if let Some(downloads) = lib.downloads {
                if let Some(artifact) = downloads.artifact {
                    let local_path = self.get_libraries_dir().join(&artifact.path);

                    if !local_path.exists() {
                        if let Some(parent) = local_path.parent() {
                            fs::create_dir_all(parent)?;
                        }

                        println!("Downloading {} -> {:?}", lib.name, local_path);
                        let mut resp = HTTP_CLIENT.get(&artifact.url).send().await?;
                        let mut file = tokio::fs::File::create(&local_path).await?;

                        while let Some(chunk) = resp.chunk().await? {
                            file.write_all(&chunk).await?;
                        }
                    } else {
                        println!("Already exists: {:?}", local_path);
                    }
                }
            } else {
                // Construire l'URL manuellement
                let parts: Vec<&str> = lib.name.split(':').collect();
                if parts.len() != 3 {
                    continue;
                }

                let group = parts[0].replace('.', "/");
                let artifact = parts[1];
                let version = parts[2];
                let file_name = format!("{}-{}.jar", artifact, version);
                let path = format!("{}/{}/{}/{}", group, artifact, version, file_name);

                let base_url = lib.url.unwrap_or_else(|| "https://maven.neoforged.net/releases/".to_string());
                let full_url = format!("{}{}", base_url, path);

                let local_path = self.get_libraries_dir().join(&path);
                if !local_path.exists() {
                    if let Some(parent) = local_path.parent() {
                        fs::create_dir_all(parent)?;
                    }

                    println!("Downloading {} -> {:?}", lib.name, local_path);
                    let mut resp = HTTP_CLIENT.get(&full_url).send().await?;
                    let mut file = tokio::fs::File::create(&local_path).await?;

                    while let Some(chunk) = resp.chunk().await? {
                        file.write_all(&chunk).await?;
                    }
                } else {
                    println!("Already exists: {:?}", local_path);
                }
            }
        }

        Ok(())
    }

    async fn get_latest_neoforge_version(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        // Déterminer l'URL en fonction de la version de Minecraft
        let url = if self.is_old_neoforge() {
            "https://maven.neoforged.net/releases/net/neoforged/forge/maven-metadata.xml"
        } else {
            "https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml"
        };

        // Télécharger le XML des métadonnées Maven
        let response = HTTP_CLIENT.get(url).send().await?;

        // Vérifier si la requête a réussi
        if !response.status().is_success() {
            return Err(format!("Échec de la requête API NeoForge: {}", response.status()).into());
        }

        // Récupérer le contenu XML
        let xml_content = response.text().await?;

        // Analyser le XML pour extraire la dernière version
        let latest_version = if let Some(start) = xml_content.find("<release>") {
            let start = start + 9; // Longueur de "<release>"
            if let Some(end) = xml_content[start..].find("</release>") {
                &xml_content[start..start+end]
            } else {
                return Err("Format XML invalide: balise </release> non trouvée".into());
            }
        } else {
            return Err("Format XML invalide: balise <release> non trouvée".into());
        };

        println!("[NeoForgeLoader] Dernière version disponible: {}", latest_version);

        Ok(latest_version.to_string())
    }

    fn is_old_neoforge(&self) -> bool {
        // Versions 1.20.1 et antérieures sont considérées comme "old" NeoForge
        version_compare::compare_to(&self.minecraft_version, "1.20.1", version_compare::Cmp::Le).unwrap_or(false)
    }

    fn get_version_id(&self) -> String {
        if self.is_old_neoforge() {
            // Format pour les anciennes versions: "1.20.1-forge-47.1.0"
            format!("forge-{}-{}", self.minecraft_version, self.loader_version)
        } else {
            // Format pour les nouvelles versions: "neoforge-20.4.72"
            format!("neoforge-{}", self.loader_version)
        }
    }
}

impl<'a> Version<'a> {
    async fn download_and_run_installer(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let version_id = self.get_version_id();

        // Assurez-vous que le répertoire existe
        if !self.get_game_dir().exists() {
            fs::create_dir_all(&self.get_game_dir())?;
        }

        // URL correcte pour l'installateur NeoForge
        let installer_url = if self.is_old_neoforge() {
            format!("https://maven.neoforged.net/releases/net/neoforged/forge/{}-{}/{}-installer.jar",
                    self.minecraft_version,self.loader_version, version_id)
        } else {
            format!("https://maven.neoforged.net/releases/net/neoforged/neoforge/{}-{}/{}-installer.jar",
                    self.minecraft_version,self.loader_version, version_id)
        };

        let installer_path = self.get_game_dir().join(format!("{}-installer.jar", version_id));

        // Télécharger l'installateur
        println!("Downloading NeoForge installer from: {}", installer_url);
        let response = HTTP_CLIENT.get(&installer_url).send().await?;

        if !response.status().is_success() {
            return Err(format!("Failed to download NeoForge installer: HTTP {}", response.status()).into());
        }

        let mut resp = response;
        let mut file = tokio::fs::File::create(&installer_path).await?;
        while let Some(chunk) = resp.chunk().await? {
            file.write_all(&chunk).await?;
        }

        // Exécuter l'installateur
        println!("Running NeoForge installer...");
        let status = Command::new("java")
            .arg("-jar")
            .arg(&installer_path)
            .arg("--fat")
            .arg(" --fat-include-minecraft-lib")
            .status()?;


        if !status.success() {
            return Err(format!("Installation de NeoForge a échoué avec le code: {:?}", status.code()).into());
        }

        // Copier le fichier JSON de configuration
        let source_json = self.get_game_dir()
            .join("versions")
            .join(&version_id)
            .join(format!("{}.json", version_id));

        // Vérifier si le fichier JSON existe avant de le copier
        if !source_json.exists() {
            return Err(format!("JSON file not found at expected location: {:?}", source_json).into());
        }

        let dest_json = self.get_game_dir().join(format!("{}.json", version_id));

        fs::copy(source_json, dest_json)?;

        // Supprimer l'installateur
        fs::remove_file(installer_path)?;

        println!("NeoForge installation completed successfully");
        Ok(())
    }
}