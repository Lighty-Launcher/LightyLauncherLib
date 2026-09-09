// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Mod (`mods/*.jar`) installation.

use lighty_loaders::types::{version_metadata::Mods, VersionInfo};

use crate::errors::InstallerResult;
use crate::installer::downloader::DownloadTask;

#[cfg(feature = "events")]
use lighty_event::EventBus;

use super::asset_partition;

/// Collects mods that need to be downloaded. Filters entries whose
/// `path` is under `mods/` (or unqualified, for legacy compat). Returns
/// the download tasks.
pub async fn collect_mod_tasks<'a>(
    version: &impl VersionInfo,
    mods: &'a [Mods],
) -> Vec<DownloadTask<'a>> {
    asset_partition::collect(version, mods, "mods", true).await
}

/// Downloads mods from pre-collected tasks. Per-file progress is
/// already surfaced through the global `LaunchEvent::InstallProgress`
/// stream, so no bucket-scoped completion event is emitted here.
pub async fn download_mods(
    tasks: Vec<DownloadTask<'_>>,
    #[cfg(feature = "events")] event_bus: Option<&EventBus>,
) -> InstallerResult<()> {
    asset_partition::download(
        tasks,
        "mods",
        #[cfg(feature = "events")]
        event_bus,
    )
    .await
}
