
use std::io::Cursor;
use std::path::{Path, PathBuf};
use anyhow::{bail, Result};
use path_absolutize::Absolutize;
use tokio::fs;

use crate::utils::system::{ OperatingSystem, OS};
use crate::utils::download::{download_file};
use crate::utils::extract::{tar_gz_extract, zip_extract} ;

use super::JavaDistribution;

/// Find java binary in JRE folder
pub async fn find_java_binary(
    runtimes_folder: &Path,
    jre_distribution: &JavaDistribution,
    jre_version: &u32,
) -> anyhow::Result<PathBuf> {
    let runtime_path =
        runtimes_folder.join(format!("{}_{}", jre_distribution.get_name(), jre_version));
    println!("runtime path: {:?}", runtime_path);
    
    //TODO: REMOVE PRINTLN

    // Find JRE in runtime folder
    let mut files = fs::read_dir(&runtime_path).await?;
    println!("Found {:?} in runtime", runtime_path);

    if let Some(jre_folder) = files.next_entry().await? {
        let folder_path = jre_folder.path();
        
        println!("Found {:?} in runtime", folder_path);

        let java_binary = match OS {
            OperatingSystem::WINDOWS => folder_path.join("bin").join("javaw.exe"),
            OperatingSystem::OSX => folder_path
                .join("Contents")
                .join("Home")
                .join("bin")
                .join("java"),
            _ => folder_path.join("bin").join("java"),
        };

        if java_binary.exists() {
            // Check if the binary has execution permissions on linux and macOS
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;

                let metadata = fs::metadata(&java_binary).await?;

                if !metadata.permissions().mode() & 0o111 != 0 {
                    // try to change permissions
                    let mut permissions = metadata.permissions();
                    permissions.set_mode(0o111);
                    fs::set_permissions(&java_binary, permissions).await?;
                }
            }

            return Ok(java_binary.absolutize()?.to_path_buf());
        }
    }

    Err(anyhow::anyhow!("Failed to find JRE"))
}

/// Download specific JRE to runtimes
pub async fn jre_download<F>(
    runtimes_folder: &Path,
    jre_distribution: &JavaDistribution,
    jre_version: &u32,
    on_progress: F,
) -> Result<PathBuf>
where
    F: Fn(u64, u64),
{
    let runtime_path =
        runtimes_folder.join(format!("{}_{}", jre_distribution.get_name(), jre_version));

    if runtime_path.exists() {
        fs::remove_dir_all(&runtime_path).await?;
    }
    fs::create_dir_all(&runtime_path).await?;

    let url = jre_distribution.get_url(jre_version)?;
    let retrieved_bytes = download_file(&url, on_progress).await?;
    let cursor = Cursor::new(&retrieved_bytes[..]);

    match OS {
        OperatingSystem::WINDOWS => zip_extract(cursor, runtime_path.as_path()).await?,
        OperatingSystem::LINUX | OperatingSystem::OSX => {
            tar_gz_extract(cursor, runtime_path.as_path()).await?
        }
        _ => bail!("Unsupported OS"),
    }

    // Find JRE afterwards
    find_java_binary(runtimes_folder, jre_distribution, jre_version).await
}
