// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! JRE download and extraction.

use std::io::Cursor;
use std::path::{Path, PathBuf};
use crate::errors::{JreError, JreResult};
use path_absolutize::Absolutize;
use tokio::fs;

use lighty_core::system::{OperatingSystem, OS};
use lighty_core::download::download_file;
use lighty_core::extract::{tar_gz_extract, zip_extract};

use super::JavaDistribution;

#[cfg(feature = "events")]
use lighty_event::{EventBus, Event, JavaEvent};

/// Locates an existing Java binary in the runtime directory.
///
/// Automatically uses a fallback distribution for unsupported
/// version/platform combinations.
pub async fn find_java_binary(
    runtimes_folder: &Path,
    distribution: &JavaDistribution,
    version: &u8,
) -> JreResult<PathBuf> {
    let effective_distribution = distribution
        .get_fallback(*version)
        .unwrap_or_else(|| distribution.clone());

    let runtime_dir = build_runtime_path(runtimes_folder, &effective_distribution, version);

    let binary_path = locate_binary_in_directory(&runtime_dir).await?;

    #[cfg(unix)]
    ensure_executable_permissions(&binary_path).await?;

    Ok(binary_path.absolutize()?.to_path_buf())
}

/// Downloads and installs a JRE to `runtimes_folder` (events feature).
#[cfg(feature = "events")]
pub async fn jre_download<F>(
    runtimes_folder: &Path,
    distribution: &JavaDistribution,
    version: &u8,
    on_progress: F,
    event_bus: Option<&EventBus>,
) -> JreResult<PathBuf>
where
    F: Fn(u64, u64),
{
    let effective_distribution = distribution
        .get_fallback(*version)
        .unwrap_or_else(|| distribution.clone());

    let runtime_dir = build_runtime_path(runtimes_folder, &effective_distribution, version);

    prepare_installation_directory(&runtime_dir).await?;

    let download_url = effective_distribution
        .get_download_url(version)
        .await
        .map_err(|e| JreError::Download(format!("Failed to get download URL: {}", e)))?;

    if let Some(bus) = event_bus {
        let response = lighty_core::hosts::HTTP_CLIENT
            .get(&download_url)
            .send()
            .await
            .map_err(|e| JreError::Download(format!("Failed to check file size: {}", e)))?;

        let total_bytes = response.content_length().unwrap_or(0);

        bus.emit(Event::Java(JavaEvent::JavaDownloadStarted {
            distribution: effective_distribution.get_name().to_string(),
            version: *version,
            total_bytes,
        }));
    }

    let archive_bytes = {
        let event_bus_ref = event_bus;
        download_file(&download_url, |current, _total| {
            on_progress(current, _total);
            if let Some(bus) = event_bus_ref {
                // Skip the initial 0 chunk
                if current > 0 {
                    bus.emit(Event::Java(JavaEvent::JavaDownloadProgress {
                        bytes: current,
                    }));
                }
            }
        })
        .await
        .map_err(|e| JreError::Download(format!("Download failed: {}", e)))?
    };

    if let Some(bus) = event_bus {
        bus.emit(Event::Java(JavaEvent::JavaDownloadCompleted {
            distribution: effective_distribution.get_name().to_string(),
            version: *version,
        }));
    }

    if let Some(bus) = event_bus {
        bus.emit(Event::Java(JavaEvent::JavaExtractionStarted {
            distribution: effective_distribution.get_name().to_string(),
            version: *version,
        }));
    }

    extract_archive(
        &archive_bytes,
        &runtime_dir,
        event_bus,
    ).await?;

    let binary_path = find_java_binary(runtimes_folder, &effective_distribution, version).await?;

    if let Some(bus) = event_bus {
        bus.emit(Event::Java(JavaEvent::JavaExtractionCompleted {
            distribution: effective_distribution.get_name().to_string(),
            version: *version,
            binary_path: binary_path.to_string_lossy().to_string(),
        }));
    }

    Ok(binary_path)
}

/// Downloads and installs a JRE to `runtimes_folder`.
#[cfg(not(feature = "events"))]
pub async fn jre_download<F>(
    runtimes_folder: &Path,
    distribution: &JavaDistribution,
    version: &u8,
    on_progress: F,
) -> JreResult<PathBuf>
where
    F: Fn(u64, u64),
{
    let effective_distribution = distribution
        .get_fallback(*version)
        .unwrap_or_else(|| distribution.clone());

    let runtime_dir = build_runtime_path(runtimes_folder, &effective_distribution, version);

    prepare_installation_directory(&runtime_dir).await?;

    let download_url = effective_distribution
        .get_download_url(version)
        .await
        .map_err(|e| JreError::Download(format!("Failed to get download URL: {}", e)))?;

    let archive_bytes = download_file(&download_url, on_progress)
        .await
        .map_err(|e| JreError::Download(format!("Download failed: {}", e)))?;

    extract_archive(&archive_bytes, &runtime_dir).await?;

    find_java_binary(runtimes_folder, &effective_distribution, version).await
}

/// Constructs the runtime installation path for a given distribution and version
fn build_runtime_path(
    runtimes_folder: &Path,
    distribution: &JavaDistribution,
    version: &u8,
) -> PathBuf {
    let mut path = runtimes_folder.to_path_buf();
    path.push(format!("{}_{}", distribution.get_name(), version));
    path
}

/// Prepares the installation directory by removing existing files
async fn prepare_installation_directory(runtime_dir: &Path) -> JreResult<()> {
    if runtime_dir.exists() {
        fs::remove_dir_all(runtime_dir).await?;
    }
    fs::create_dir_all(runtime_dir).await?;
    Ok(())
}

/// Extracts the JRE archive based on the operating system (events feature).
#[cfg(feature = "events")]
async fn extract_archive(
    archive_bytes: &[u8],
    destination: &Path,
    event_bus: Option<&EventBus>,
) -> JreResult<()> {
    let cursor = Cursor::new(archive_bytes);

    match OS {
        OperatingSystem::WINDOWS => {
            zip_extract(cursor, destination, event_bus)
                .await
                .map_err(|e| JreError::Extraction(format!("ZIP extraction failed: {}", e)))?;
        }
        OperatingSystem::LINUX | OperatingSystem::OSX => {
            tar_gz_extract(cursor, destination, event_bus)
                .await
                .map_err(|e| JreError::Extraction(format!("TAR.GZ extraction failed: {}", e)))?;
        }
        OperatingSystem::UNKNOWN => {
            return Err(JreError::UnsupportedOS);
        }
    }

    Ok(())
}

/// Extracts the JRE archive based on the operating system.
#[cfg(not(feature = "events"))]
async fn extract_archive(archive_bytes: &[u8], destination: &Path) -> JreResult<()> {
    let cursor = Cursor::new(archive_bytes);

    match OS {
        OperatingSystem::WINDOWS => {
            zip_extract(cursor, destination)
                .await
                .map_err(|e| JreError::Extraction(format!("ZIP extraction failed: {}", e)))?;
        }
        OperatingSystem::LINUX | OperatingSystem::OSX => {
            tar_gz_extract(cursor, destination)
                .await
                .map_err(|e| JreError::Extraction(format!("TAR.GZ extraction failed: {}", e)))?;
        }
        OperatingSystem::UNKNOWN => {
            return Err(JreError::UnsupportedOS);
        }
    }

    Ok(())
}

/// Locates the java binary within the extracted JRE directory.
///
/// Structure varies by OS and distribution:
/// - Windows: jre_root/bin/java.exe
/// - macOS (bundle): jre_root/Contents/Home/bin/java (Temurin)
/// - macOS (nested bundle): jre_root/*.jre/Contents/Home/bin/java (Zulu Java 8)
/// - macOS (flat): jre_root/bin/java (Liberica tar.gz)
/// - Linux: jre_root/bin/java
async fn locate_binary_in_directory(runtime_dir: &Path) -> JreResult<PathBuf> {
    let mut entries = fs::read_dir(runtime_dir).await?;

    let jre_root = entries
        .next_entry()
        .await?
        .ok_or_else(|| JreError::NotFound {
            path: runtime_dir.to_path_buf(),
        })?
        .path();

    let java_binary = match OS {
        OperatingSystem::WINDOWS => jre_root.join("bin").join("java.exe"),
        OperatingSystem::OSX => {
            // Try direct bundle, then nested .jre bundle (Zulu Java 8), then flat (Liberica tar.gz).
            let bundle_path = jre_root.join("Contents").join("Home").join("bin").join("java");
            if bundle_path.exists() {
                bundle_path
            }
            else if let Some(nested) = find_nested_jre_bundle(&jre_root).await {
                nested
            }
            else {
                jre_root.join("bin").join("java")
            }
        }
        _ => jre_root.join("bin").join("java"),
    };

    if !java_binary.exists() {
        return Err(JreError::NotFound {
            path: java_binary.clone(),
        });
    }

    Ok(java_binary)
}

/// Finds a nested .jre bundle inside the JRE root (Zulu Java 8 on macOS)
#[cfg(target_os = "macos")]
async fn find_nested_jre_bundle(jre_root: &Path) -> Option<PathBuf> {
    let mut entries = fs::read_dir(jre_root).await.ok()?;

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name()?.to_str()?;
            if name.ends_with(".jre") {
                let java_path = path.join("Contents").join("Home").join("bin").join("java");
                if java_path.exists() {
                    return Some(java_path);
                }
            }
        }
    }
    None
}

#[cfg(not(target_os = "macos"))]
async fn find_nested_jre_bundle(_jre_root: &Path) -> Option<PathBuf> {
    None
}

/// Ensures the java binary has execution permissions on Unix systems
#[cfg(unix)]
async fn ensure_executable_permissions(binary_path: &Path) -> JreResult<()> {
    use std::os::unix::fs::PermissionsExt;

    let metadata = fs::metadata(binary_path).await?;
    let current_permissions = metadata.permissions();

    if current_permissions.mode() & 0o111 == 0 {
        let mut new_permissions = current_permissions;
        new_permissions.set_mode(0o755);
        fs::set_permissions(binary_path, new_permissions).await?;
    }

    Ok(())
}
