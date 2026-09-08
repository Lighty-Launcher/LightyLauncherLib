// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Assets installation.

use lighty_loaders::types::{VersionInfo, version_metadata::AssetsFile};
use lighty_core::time_it;
use crate::errors::InstallerResult;
use crate::installer::verifier::{is_missing_or_empty, needs_download};
use crate::installer::downloader::download_small_with_concurrency_limit;

#[cfg(feature = "events")]
use lighty_event::EventBus;

/// Collects assets that need to be downloaded.
pub async fn collect_asset_tasks(
    version: &impl VersionInfo,
    assets: Option<&AssetsFile>,
) -> Vec<(String, std::path::PathBuf, String)> {
    let Some(assets) = assets else {
        return Vec::new();
    };

    let assets_root = version.game_dirs().join("assets");
    let objects_root = assets_root.join("objects");
    let mut tasks = Vec::new();

    for asset in assets.objects.values() {
        let Some(url) = &asset.url else { continue };

        // A custom path is not content-addressed, so its content still has
        // to be hashed; everything under objects/ is named by its own SHA1.
        let outdated = match &asset.path {
            Some(custom) => {
                let path = assets_root.join(custom);
                needs_download(&path, Some(&asset.hash), &asset.hash)
                    .await
                    .then_some(path)
            }
            None => {
                let path = objects_root.join(&asset.hash[0..2]).join(&asset.hash);
                is_missing_or_empty(&path).await.then_some(path)
            }
        };

        if let Some(path) = outdated {
            tasks.push((url.clone(), path, asset.hash.clone()));
        }
    }

    // Two index entries can share a hash and thus one destination file:
    // 1.7.x declares every sound under both `sound/` and `sounds/`.
    tasks.sort_unstable_by(|left, right| left.1.cmp(&right.1));
    tasks.dedup_by(|left, right| left.1 == right.1);

    tasks
}

/// Downloads assets from pre-collected tasks.
pub async fn download_assets(
    tasks: Vec<(String, std::path::PathBuf, String)>,
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
