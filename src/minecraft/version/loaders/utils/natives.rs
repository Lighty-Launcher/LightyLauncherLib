use std::error::Error;
use std::path::{Path, PathBuf};
use std::collections::HashSet;
use serde_json::Value;
use tokio::fs;
use crate::minecraft::version::loaders::utils::manifest::Manifest;
use crate::minecraft::version::version::Version;
use crate::utils::system::{OS, ARCHITECTURE, OperatingSystem, Architecture};
use super::download::{download_file, should_download_library};
pub trait Natives<'a> {
    async fn download_natives(&self) -> Result<(), Box<dyn Error + Send + Sync>>;
}
impl<'a> Natives<'a> for Version<'a> {
    async fn download_natives(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let version_data = self.get_manifest_version().await?;
        let natives_dir = &self.get_natives_dir();
        let libraries = match version_data["libraries"].as_array() {
            Some(libs) => libs,
            None => return Err("Libraries not found in version data".into()),
        };

        println!("[LightyLauncher] Downloading native libraries...");
        let mut downloaded = 0;

        // Create the natives directory if it doesn't exist
        if !natives_dir.exists() {
            fs::create_dir_all(natives_dir).await?;
        }

        // Count natives first
        let mut native_count = 0;
        for library in libraries.iter() {
            if !should_download_library(library) {
                continue;
            }

            // Check if this library has natives for our platform
            if let Some(natives_obj) = library.get("natives") {
                let os_key = match OS {
                    OperatingSystem::WINDOWS => "windows",
                    OperatingSystem::OSX => "osx",
                    OperatingSystem::LINUX => "linux",
                    _ => continue, // Skip if OS not supported
                };

                if natives_obj.get(os_key).is_some() {
                    native_count += 1;
                }
            }
        }

        println!("[LightyLauncher] Found {} native libraries to check", native_count);

        // Now download each native
        let mut current = 0;
        for library in libraries.iter() {
            if !should_download_library(library) {
                continue;
            }

            // Get library name for logging
            let lib_name = library["name"].as_str().unwrap_or("unknown");

            // Check if this library has natives for our platform
            if let Some(natives_obj) = library.get("natives") {
                // Determine which native key to use based on OS
                let os_key = match OS {
                    OperatingSystem::WINDOWS => "windows",
                    OperatingSystem::OSX => "osx",
                    OperatingSystem::LINUX => "linux",
                    _ => continue, // Skip if OS not supported
                };

                if let Some(native_value) = natives_obj.get(os_key) {
                    current += 1;

                    // Format the native classifier key (handle ${arch} replacement)
                    let native_classifier = match native_value.as_str() {
                        Some(val) => {
                            let mut classifier = val.to_string();
                            if classifier.contains("${arch}") {
                                let arch_value = match ARCHITECTURE {
                                    Architecture::X86 => "32",
                                    Architecture::X64 => "64",
                                    _ => "64", // Default to 64-bit
                                };
                                classifier = classifier.replace("${arch}", arch_value);
                            }
                            classifier
                        },
                        None => continue, // Skip if no string value
                    };

                    println!("[LightyLauncher] Processing native: {} ({}/{})", lib_name, current, native_count);

                    // Look for the classifier in the downloads section
                    if let Some(downloads) = library.get("downloads") {
                        if let Some(classifiers) = downloads.get("classifiers") {
                            if let Some(native_info) = classifiers.get(&native_classifier) {
                                // Extract download information
                                let url = native_info["url"].as_str().ok_or(format!("Native URL not found for {}", lib_name))?;
                                let sha1 = native_info["sha1"].as_str().ok_or(format!("Native SHA1 not found for {}", lib_name))?;
                                let size = native_info["size"].as_u64().ok_or(format!("Native size not found for {}", lib_name))?;
                                let path = native_info["path"].as_str().ok_or(format!("Native path not found for {}", lib_name))?;

                                // Create a filename from the path
                                let file_name = Path::new(path)
                                    .file_name()
                                    .ok_or(format!("Cannot extract filename from {}", path))?;

                                let native_path = natives_dir.clone().join(file_name);

                                // Download if not exists
                                if !native_path.exists() {
                                    println!("[LightyLauncher] Downloading native: {} ({}/{})", lib_name, current, native_count);
                                    download_file(url, &native_path, sha1, size).await?;
                                    downloaded += 1;

                                    // Extract the native if it's a JAR
                                    if path.ends_with(".jar") {
                                        println!("[LightyLauncher] Extracting native: {}", lib_name);
                                        extract_native(&native_path, &natives_dir.clone(), library).await?;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        println!("[LightyLauncher] Downloaded and processed {} new native libraries", downloaded);
        Ok(())
    }


}

// Helper method to extract native libraries
async fn extract_native( jar_path: &Path, natives_dir: &Path, library: &Value) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Lire le fichier JAR entièrement de manière asynchrone d'abord
    let file_data = fs::read(jar_path).await?;

    // Convertir natives_dir en PathBuf pour qu'il soit possédé par la closure
    let natives_dir_owned = natives_dir.to_path_buf();

    // Cloner la valeur library pour qu'elle soit possédée par la closure
    let library_owned = library.clone();

    // Puis traiter l'archive ZIP de façon synchrone dans une tâche de blocage
    let files_to_write = tokio::task::spawn_blocking(move || -> Result<Vec<(PathBuf, Option<Vec<u8>>)>, Box<dyn Error + Send + Sync>> {
        // Utiliser un Cursor avec les données lues pour créer une archive ZIP
        let cursor = std::io::Cursor::new(file_data);
        let mut archive = zip::ZipArchive::new(cursor)?;

        // Get exclude patterns from the library metadata
        let mut exclude_patterns = HashSet::new();
        if let Some(extract) = library_owned.get("extract") {
            if let Some(exclude) = extract.get("exclude") {
                if let Some(exclude_array) = exclude.as_array() {
                    for pattern in exclude_array {
                        if let Some(pattern_str) = pattern.as_str() {
                            exclude_patterns.insert(pattern_str.to_string());
                        }
                    }
                }
            }
        }

        // Préparer les données à extraire
        let mut files_to_write = Vec::new();

        // Extract files, excluding those that match patterns
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let file_path = match file.enclosed_name() {
                Some(path) => path.to_owned(),
                None => continue,
            };

            let file_path_str = file_path.to_string_lossy();

            // Skip excluded files
            let mut skip = false;
            for pattern in &exclude_patterns {
                if file_path_str.starts_with(pattern) || file_path_str.contains(pattern) {
                    skip = true;
                    break;
                }
            }

            if skip {
                continue;
            }

            let output_path = natives_dir_owned.join(&file_path);

            if file.is_dir() {
                // Juste mémoriser les répertoires à créer
                files_to_write.push((output_path, None));
            } else {
                // Lire le contenu du fichier
                let mut buffer = Vec::new();
                std::io::copy(&mut file, &mut buffer)?;

                // Mémoriser le fichier et son contenu
                files_to_write.push((output_path, Some(buffer)));
            }
        }

        Ok(files_to_write)
    }).await??;

    // Maintenant, écrire les fichiers de manière asynchrone
    for (path, content_opt) in files_to_write {
        if let Some(content) = content_opt {
            // C'est un fichier
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent).await?;
                }
            }
            fs::write(&path, content).await?;
        } else {
            // C'est un répertoire
            fs::create_dir_all(&path).await?;
        }
    }
    // Supprimer le fichier JAR après extraction
    fs::remove_file(jar_path).await?;

    Ok(())
}