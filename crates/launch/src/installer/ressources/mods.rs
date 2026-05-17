// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Mod (`mods/*.jar`) installation.

use std::path::PathBuf;

use lighty_loaders::types::{version_metadata::Mods, VersionInfo};

use crate::errors::InstallerResult;

#[cfg(feature = "events")]
use lighty_event::EventBus;

use super::asset_partition;

/// Collects mods that need to be downloaded. Filters entries whose
/// `path` is under `mods/` (or unqualified, for legacy compat). Returns
/// `(tasks, total_bytes_to_download)`.
pub async fn collect_mod_tasks(
    version: &impl VersionInfo,
    mods: &[Mods],
) -> (Vec<(String, PathBuf)>, u64) {
    asset_partition::collect(version, mods, "mods", true).await
}

/// Downloads mods from pre-collected tasks. Per-file progress is
/// already surfaced through the global `LaunchEvent::InstallProgress`
/// stream, so no bucket-scoped completion event is emitted here.
pub async fn download_mods(
    tasks: Vec<(String, PathBuf)>,
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
