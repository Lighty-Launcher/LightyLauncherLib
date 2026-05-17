// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Library installation.

use lighty_loaders::types::{VersionInfo, version_metadata::Library};
use lighty_core::time_it;
use crate::errors::InstallerResult;
use crate::installer::verifier::needs_download;
use crate::installer::downloader::download_with_concurrency_limit;

#[cfg(feature = "events")]
use lighty_event::EventBus;

/// Collects libraries that need to be downloaded.
pub async fn collect_library_tasks(
    version: &impl VersionInfo,
    libraries: &[Library],
) -> Vec<(String, std::path::PathBuf)> {
    let parent_path = version.game_dirs().join("libraries");
    let mut tasks = Vec::new();

    for lib in libraries {
        let Some(url) = &lib.url else { continue };
        // Forge-family installers list libraries with empty URLs when the file
        // is produced by post-install processors or bundled inside the installer
        // JAR; reqwest can't build a request from an empty URL.
        if url.is_empty() {
            continue;
        }
        let Some(path_str) = &lib.path else { continue };

        let path = parent_path.join(path_str);

        if needs_download(&path, lib.sha1.as_ref(), &lib.name).await {
            tasks.push((url.clone(), path));
        }
    }

    tasks
}

/// Downloads libraries from pre-collected tasks.
pub async fn download_libraries(
    tasks: Vec<(String, std::path::PathBuf)>,
    #[cfg(feature = "events")] event_bus: Option<&EventBus>,
) -> InstallerResult<()> {
    if tasks.is_empty() {
        lighty_core::trace_info!("[Installer] All libraries already cached and verified");
        return Ok(());
    }

    lighty_core::trace_info!("[Installer] Downloading {} libraries...", tasks.len());
    time_it!("Libraries download", {
        download_with_concurrency_limit(
            tasks,
            #[cfg(feature = "events")]
            event_bus,
        )
        .await?
    });
    lighty_core::trace_info!("[Installer] Libraries installed");
    Ok(())
}
