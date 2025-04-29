use std::error::Error;
use std::path::Path;
use crate::minecraft::version::loaders::utils::manifest::Manifest;
use crate::minecraft::version::version::Version;
use crate::utils::system::OS;
use super::download::{download_file, should_download_library};

pub trait Libraries<'a> {
    async fn download_libraries(&self) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn get_all_libraries_dir(&self) -> Result<String, Box<dyn Error + Send + Sync>>;
}
impl<'a> Libraries<'a> for Version<'a> {
    async fn download_libraries(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let version_data = self.get_manifest_version().await?;
        let libraries = match version_data["libraries"].as_array() {
            Some(libs) => libs,
            None => return Err("Libraries not found in version data".into()),
        };

        println!("[LightyLauncher] Downloading libraries...");
        let mut downloaded = 0;
        let total = libraries.len();

        for (index, library) in libraries.iter().enumerate() {
            if !should_download_library(library) {
                continue;
            }

            if let Some(downloads) = library["downloads"].as_object() {
                if let Some(artifact) = downloads.get("artifact") {
                    let path_str = artifact["path"].as_str().ok_or("Library path not found")?;
                    let url = artifact["url"].as_str().ok_or("Library URL not found")?;
                    let sha1 = artifact["sha1"].as_str().ok_or("Library SHA1 not found")?;
                    let size = artifact["size"].as_u64().ok_or("Library size not found")?;

                    let lib_path = self.get_libraries_dir().join(path_str);

                    if !lib_path.exists() {
                        if let Some(parent) = lib_path.parent() {
                            tokio::fs::create_dir_all(parent).await?;
                        }

                        download_file(url, &lib_path, sha1, size).await?;
                        downloaded += 1;
                    }
                }
            }

            if (index + 1) % 5 == 0 || index == total - 1 {
                println!("[LightyLauncher] Libraries progress: {}/{}", index + 1, total);
            }
        }

        println!("[LightyLauncher] Downloaded {} new libraries", downloaded);
        Ok(())
    }
    async fn get_all_libraries_dir(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        // Obtenir le chemin de base des bibliothèques
        let base_dir = self.get_libraries_dir();
        let mut paths = Vec::new();

        // Fonction récursive pour parcourir les dossiers
        fn collect_jar_files(dir: &Path, paths: &mut Vec<String>) -> Result<(), Box<dyn Error + Send + Sync>> {
            if dir.is_dir() {
                for entry in std::fs::read_dir(dir)? {
                    let entry = entry?;
                    let path = entry.path();

                    if path.is_dir() {
                        // Récursion dans les sous-dossiers
                        collect_jar_files(&path, paths)?;
                    } else if path.is_file() {
                        // Vérifier si c'est un fichier JAR
                        if let Some(extension) = path.extension() {
                            if extension == "jar" {
                                paths.push(path.to_string_lossy().to_string());
                            }
                        }
                    }
                }
            }
            Ok(())
        }
        // Lancer la récursion à partir du dossier de base
        collect_jar_files(&base_dir, &mut paths)?;

        //TODO! check if is not infinite loop
        if paths.is_empty() {
            self.download_libraries().await?;
            collect_jar_files(&base_dir, &mut paths)?;
        }

        // Détecter le séparateur de chemin selon l'OS
        let sep = OS.get_path_separator()?; // retourne ";" ou ":" en fonction de l'OS
        Ok(paths.join(sep))
    }
}

