// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Native libraries installation and extraction.

use std::path::PathBuf;
use async_zip::tokio::read::seek::ZipFileReader;
use tokio::fs;
use tokio::io::BufReader;
use lighty_loaders::types::{VersionInfo, version_metadata::Native};
use lighty_core::{mkdir, time_it};
use futures::future::try_join_all;
use futures_util::io;
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
use crate::errors::{InstallerError, InstallerResult};
use crate::installer::verifier::needs_download;
use crate::installer::downloader::download_with_concurrency_limit;

#[cfg(feature = "events")]
use lighty_event::EventBus;

/// Collects natives that need downloading and paths for extraction.
pub async fn collect_native_tasks(
    version: &impl VersionInfo,
    natives: &[Native],
) -> (Vec<(String, PathBuf)>, Vec<PathBuf>) {
    if natives.is_empty() {
        return (Vec::new(), Vec::new());
    }

    let libraries_path = version.game_dirs().join("libraries");
    let mut download_tasks = Vec::new();
    let mut extract_paths = Vec::new();

    for native in natives {
        let Some(url) = &native.url else { continue };
        let Some(path_str) = &native.path else { continue };

        let jar_path = libraries_path.join(path_str);

        if needs_download(&jar_path, native.sha1.as_ref(), &native.name).await {
            download_tasks.push((url.clone(), jar_path.clone()));
        }

        extract_paths.push(jar_path);
    }

    (download_tasks, extract_paths)
}

/// Downloads and extracts natives from pre-collected tasks.
pub async fn download_and_extract_natives(
    version: &impl VersionInfo,
    download_tasks: Vec<(String, PathBuf)>,
    extract_paths: Vec<PathBuf>,
    #[cfg(feature = "events")] event_bus: Option<&EventBus>,
) -> InstallerResult<()> {
    let natives_extract_path = version.game_dirs().join("natives");

    // Natives are cleaned on each install since LWJGL needs a fresh extraction.
    if natives_extract_path.exists() {
        let _ = fs::remove_dir_all(&natives_extract_path).await;
    }
    mkdir!(natives_extract_path);

    if !download_tasks.is_empty() {
        lighty_core::trace_info!("[Installer] Downloading {} natives...", download_tasks.len());
        time_it!("Natives download", {
            download_with_concurrency_limit(
                download_tasks,
                #[cfg(feature = "events")]
                event_bus,
            )
            .await?
        });
        lighty_core::trace_info!("[Installer] Natives downloaded");
    } else {
        lighty_core::trace_info!("[Installer] All natives already cached and verified");
    }

    if !extract_paths.is_empty() {
        lighty_core::trace_info!("[Installer] Extracting {} natives...", extract_paths.len());
        let extraction_tasks: Vec<_> = extract_paths
            .into_iter()
            .map(|jar_path| extract_native(jar_path, natives_extract_path.clone()))
            .collect();

        time_it!("Natives extraction", try_join_all(extraction_tasks).await?);
        lighty_core::trace_info!("[Installer] Natives extracted");
    }

    Ok(())
}

/// Extracts a native JAR using async ZIP extraction.
async fn extract_native(jar_path: PathBuf, natives_dir: PathBuf) -> InstallerResult<()> {
    let file = tokio::fs::File::open(&jar_path).await?;
    let buffered = BufReader::new(file);
    let mut reader = ZipFileReader::new(buffered.compat()).await?;

    let entries_count = reader.file().entries().len();

    for index in 0..entries_count {
        // Collect entry metadata before mutably borrowing reader.
        let (file_name, should_extract) = {
            let entry = reader.file().entries().get(index)
                .ok_or_else(|| InstallerError::MissingField(
                    format!("ZIP entry {} not found in {}", index, jar_path.display())
                ))?;

            let file_name = entry.filename().as_str()?.to_string();
            let should_extract = is_native_file(&file_name);

            (file_name, should_extract)
        };

        if should_extract {
            let dest_path = natives_dir.join(
                std::path::Path::new(&file_name)
                    .file_name()
                    .unwrap_or_default()
            );

            let mut entry_reader = reader.reader_with_entry(index).await?;
            let dest_file = tokio::fs::File::create(&dest_path).await?;

            io::copy(&mut entry_reader, &mut dest_file.compat_write()).await?;
        }
    }

    Ok(())
}

/// Returns true for native library files inside the JAR.
///
/// Accepts standard extensions and Linux soname-versioned variants
/// (`*.so.N(.N)*`) but excludes sidecars like `.sha1`, `.git`, `.md5`.
#[inline]
fn is_native_file(filename: &str) -> bool {
    const NATIVE_EXTENSIONS: &[&str] = &[".dll", ".so", ".dylib", ".jnilib"];

    let filename_lower = filename.to_lowercase();

    if NATIVE_EXTENSIONS.iter().any(|ext| filename_lower.ends_with(ext)) {
        return true;
    }

    // Suffix after last `.so.` must be digits/dots only (`1`, `1.2.3`).
    if let Some(suffix) = filename_lower.rsplit_once(".so.").map(|(_, s)| s) {
        return !suffix.is_empty()
            && suffix.chars().all(|c| c.is_ascii_digit() || c == '.');
    }

    false
}
