use std::error::Error;
use std::path::Path;
use sha1::{Sha1, Digest};
use serde_json::Value;
use tokio::fs;
use crate::utils::system::{OS, OperatingSystem};

pub(crate) async fn download_file(url: &str, path: &Path, expected_sha1: &str, expected_size: u64) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).await?;
        }
    }

    // Download the file
    let response = reqwest::get(url).await?;

    if !response.status().is_success() {
        return Err(format!("Failed to download from {}: HTTP {}", url, response.status()).into());
    }

    let content = response.bytes().await?;

    // Verify size
    if content.len() as u64 != expected_size {
        return Err(format!(
            "Size mismatch for {}: expected {}, got {}",
            path.display(), expected_size, content.len()
        ).into());
    }

    // Verify SHA1
    let mut hasher = Sha1::new();
    hasher.update(&content);
    let hash = hasher.finalize();
    let hash_str = hex::encode(hash);

    if hash_str != expected_sha1 {
        return Err(format!(
            "SHA1 mismatch for {}: expected {}, got {}",
            path.display(), expected_sha1, hash_str
        ).into());
    }

    // Write file
    fs::write(path, content).await?;

    Ok(())
}

// Helper to determine if a library should be downloaded based on rules
pub(crate) fn should_download_library(library: &Value) -> bool {
    // If no rules exist, download the library
    if !library.get("rules").is_some() {
        return true;
    }

    let rules = match library["rules"].as_array() {
        Some(r) => r,
        None => return true, // No valid rules array means download by default
    };

    let mut allowed = false;

    for rule in rules {
        let action = rule["action"].as_str().unwrap_or("disallow");

        // Default rule with no OS specification
        if !rule.get("os").is_some() {
            allowed = action == "allow";
            continue;
        }

        // Rule with OS specification
        if let Some(os) = rule["os"].as_object() {
            let os_name = os["name"].as_str().unwrap_or("");

            let matches_os = match OS {
                OperatingSystem::WINDOWS => os_name == "windows",
                OperatingSystem::OSX => os_name == "osx",
                OperatingSystem::LINUX => os_name == "linux",
                _ => false,
            };

            // Check if rule applies to current OS
            if matches_os {
                allowed = action == "allow";
            }
        }
    }

    allowed
}