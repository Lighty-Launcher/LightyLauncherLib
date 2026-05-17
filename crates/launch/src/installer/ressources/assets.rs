// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Assets installation.

use lighty_loaders::types::{VersionInfo, version_metadata::AssetsFile};
use lighty_core::time_it;
use crate::errors::InstallerResult;
use crate::installer::verifier::needs_download;
use crate::installer::downloader::download_small_with_concurrency_limit;

#[cfg(feature = "events")]
use lighty_event::EventBus;

/// Collects assets that need to be downloaded.
pub async fn collect_asset_tasks(
    version: &impl VersionInfo,
    assets: Option<&AssetsFile>,
) -> Vec<(String, std::path::PathBuf)> {
    let Some(assets) = assets else {
        return Vec::new();
    };

    let parent_path = version.game_dirs().join("assets").join("objects");
    let mut tasks = Vec::new();

    for asset in assets.objects.values() {
        let Some(url) = &asset.url else { continue };

        // First 2 chars of hash form the subdirectory.
        let hash_prefix = &asset.hash[0..2];
        let path = parent_path.join(hash_prefix).join(&asset.hash);

        if needs_download(&path, Some(&asset.hash), &asset.hash).await {
            tasks.push((url.clone(), path));
        }
    }

    tasks
}

/// Downloads assets from pre-collected tasks.
pub async fn download_assets(
    tasks: Vec<(String, std::path::PathBuf)>,
    #[cfg(feature = "events")] event_bus: Option<&EventBus>,
) -> InstallerResult<()> {
    if tasks.is_empty() {
        lighty_core::trace_info!("[Installer] All assets already cached and verified");
        return Ok(());
    }

    lighty_core::trace_info!("[Installer] Downloading {} new assets...", tasks.len());
    time_it!("Assets download", {
        download_small_with_concurrency_limit(
            tasks,
            #[cfg(feature = "events")]
            event_bus,
        )
        .await?
    });
    lighty_core::trace_info!("[Installer] Assets installed");
    Ok(())
}
