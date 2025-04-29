use serde_json::Value;
use std::error::Error;
use std::path::Path;
use tokio::fs;
use crate::minecraft::version::loaders::utils::download::download_file;
use crate::minecraft::version::loaders::utils::manifest::Manifest;
use crate::minecraft::version::version::Version;

pub trait Assets<'a> {
    async fn download_assets(&self) -> Result<(), Box<dyn Error + Send + Sync>>;
}
impl<'a> Assets<'a> for Version<'a> {
    async fn download_assets(&self) -> Result<(), Box<dyn Error + Send + Sync>> {

        let version_data = self.get_manifest_version().await?;


        // Create directories
        let indexes_dir = self.get_assets_dir().join("indexes");
        let objects_dir = self.get_assets_dir().join("objects");

        for dir in [&indexes_dir, &objects_dir] {
            if !dir.exists() {
                fs::create_dir_all(dir).await?;
            }
        }

        // Get asset index info
        let asset_index = version_data
            .get("assetIndex")
            .and_then(|v| v.as_object());if let Some(idx) = asset_index {
            // continue avec idx...
        } else if let Some(asset_name) = version_data.get("assets").and_then(|v| v.as_str()) {
            println!("[LightyLauncher] Using legacy asset index: {}", asset_name);
            return download_legacy_assets(asset_name, &self.get_assets_dir()).await;
        } else {
            return Err("Asset index information not found".into());
        }
        ;

        if let Some(idx) = version_data
            .get("assetIndex")
            .and_then(|v| v.as_object()) {

            let id = idx.get("id")
                .and_then(|v| v.as_str())
                .ok_or("Asset index ID not found")?;

            let url = idx.get("url")
                .and_then(|v| v.as_str())
                .ok_or("Asset index URL not found")?;

            let sha1 = idx.get("sha1")
                .and_then(|v| v.as_str())
                .ok_or("Asset index SHA1 not found")?;

            let size = idx.get("size")
                .and_then(|v| v.as_u64())
                .ok_or("Asset index size not found")?;

            // ensuite continue le traitement...
            let index_path = indexes_dir.join(format!("{}.json", self.minecraft_version));
            if !index_path.exists() {
                println!("[LightyLauncher] Downloading asset index from: {}", url);
                download_file(url, &index_path, sha1, size).await?;
            }
            // Read the asset index
            let index_content = fs::read_to_string(&index_path).await?;
            let index_json: Value = serde_json::from_str(&index_content)?;

            let objects = match index_json["objects"].as_object() {
                Some(objs) => objs,
                None => return Err("Objects not found in asset index".into()),
            };

            println!("[LightyLauncher] Found {} assets to verify", objects.len());

            // Track progress
            let total = objects.len();
            let mut downloaded = 0;
            let mut current = 0;

            // Download all objects
            for (asset_name, object) in objects {
                current += 1;

                let hash = object["hash"].as_str().ok_or(format!("Hash not found for asset {}", asset_name))?;
                let hash_prefix = &hash[0..2];

                let object_path = objects_dir.join(hash_prefix).join(hash);

                if !object_path.exists() {
                    let size = object["size"].as_u64().ok_or(format!("Size not found for asset {}", asset_name))?;
                    let url = format!("https://resources.download.minecraft.net/{}/{}", hash_prefix, hash);

                    if let Some(parent) = object_path.parent() {
                        fs::create_dir_all(parent).await?;
                    }

                    download_file(&url, &object_path, hash, size).await?;
                    downloaded += 1;
                }

                // Print progress every 50 assets or for the last one
                if current % 50 == 0 || current == total {
                    println!("[LightyLauncher] Assets progress: {}/{}", current, total);
                }
            }

            println!("[LightyLauncher] Downloaded {} new assets", downloaded);

        }

        //TODO! recheck the logic of this part



        Ok(())
    }

}


async fn download_legacy_assets( asset_version: &str, assets_dir: &Path) -> Result<(), Box<dyn Error + Send + Sync>> {
    // For legacy asset structure, fetch from Mojang
    let url = format!("https://launchermeta.mojang.com/v1/packages/1863782e33ce7b584fc45b037325a1964e095d3e/{}.json", asset_version);
    let indexes_dir = assets_dir.join("indexes");
    let objects_dir = assets_dir.join("objects");

    for dir in [&indexes_dir, &objects_dir] {
        if !dir.exists() {
            fs::create_dir_all(dir).await?;
        }
    }

    let index_path = indexes_dir.join(format!("{}.json", asset_version));


    // Download asset index JSON
    println!("[LightyLauncher] Downloading legacy asset index from: {}", url);
    let index_response = reqwest::get(&url).await?;
    let index_json: Value = index_response.json().await?;

    // Save the index JSON
    fs::write(&index_path, serde_json::to_string_pretty(&index_json)?).await?;

    // Process the objects
    if let Some(objects) = index_json["objects"].as_object() {
        println!("[LightyLauncher] Found {} legacy assets to verify", objects.len());

        let total = objects.len();
        let mut downloaded = 0;
        let mut current = 0;

        for (asset_name, object) in objects {
            current += 1;

            let hash = object["hash"].as_str().ok_or(format!("Hash not found for legacy asset {}", asset_name))?;
            let hash_prefix = &hash[0..2];

            let object_path = objects_dir.join(hash_prefix).join(hash);

            if !object_path.exists() {
                let size = object["size"].as_u64().ok_or(format!("Size not found for legacy asset {}", asset_name))?;
                let url = format!("https://resources.download.minecraft.net/{}/{}", hash_prefix, hash);

                if let Some(parent) = object_path.parent() {
                    fs::create_dir_all(parent).await?;
                }

                download_file(&url, &object_path, hash, size).await?;
                downloaded += 1;
            }

            // Print progress
            if current % 50 == 0 || current == total {
                println!("[LightyLauncher] Legacy assets progress: {}/{}", current, total);
            }
        }

        println!("[LightyLauncher] Downloaded {} new legacy assets", downloaded);
    }

    Ok(())
}