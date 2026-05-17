// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Shared partition + download helpers for mod-like assets (mods,
//! resourcepacks, shaderpacks, datapacks). Each public asset module
//! is a thin wrapper around these helpers with a fixed subdir prefix.

use std::path::PathBuf;

use lighty_core::time_it;
use lighty_loaders::types::{version_metadata::Mods, VersionInfo};

use crate::errors::InstallerResult;
use crate::installer::downloader::download_with_concurrency_limit;
use crate::installer::verifier::needs_download;

#[cfg(feature = "events")]
use lighty_event::EventBus;

/// Collects download tasks for every `Mods` entry whose `path`
/// targets `subdir`. Returns `(tasks, total_bytes)` where `total_bytes`
/// is the sum of the entries' declared size — used by bucket-scoped
/// events and the global progress total.
///
/// `legacy_fallback`: when true and an entry's `path` has no `/`,
/// treat it as `<subdir>/<filename>` and emit a deprecation warn.
/// Only enabled for `subdir == "mods"` during the migration window.
pub(super) async fn collect<V: VersionInfo>(
    version: &V,
    mods: &[Mods],
    subdir: &str,
    legacy_fallback: bool,
) -> (Vec<(String, PathBuf)>, u64) {
    if mods.is_empty() {
        return (Vec::new(), 0);
    }

    let runtime = version.runtime_dir();
    let parent = runtime.join(subdir);
    lighty_core::mkdir!(&parent);

    let prefix = format!("{}/", subdir);
    let mut tasks = Vec::new();
    let mut bytes = 0u64;

    for entry in mods {
        let Some(url) = &entry.url else { continue };
        let Some(path_str) = &entry.path else { continue };

        let target = if let Some(rest) = path_str.strip_prefix(&prefix) {
            parent.join(rest)
        } else if legacy_fallback && !path_str.contains('/') && !path_str.contains('\\') {
            lighty_core::trace_warn!(
                "[Installer] Legacy unqualified path '{}' — falling back to {}/",
                path_str,
                subdir
            );
            parent.join(path_str)
        } else {
            continue;
        };

        if let Some(dir) = target.parent() {
            lighty_core::mkdir!(dir);
        }

        if needs_download(&target, entry.sha1.as_ref(), &entry.name).await {
            bytes += entry.size.unwrap_or(0);
            tasks.push((url.clone(), target));
        }
    }

    (tasks, bytes)
}

/// Downloads a partitioned task batch with a human-readable label
/// used for logging (`"mods"`, `"resourcepacks"`, ...).
pub(super) async fn download(
    tasks: Vec<(String, PathBuf)>,
    label: &str,
    #[cfg(feature = "events")] event_bus: Option<&EventBus>,
) -> InstallerResult<()> {
    if tasks.is_empty() {
        return Ok(());
    }

    lighty_core::trace_info!("[Installer] Downloading {} {}...", tasks.len(), label);
    let label_owned = format!("{} download", label);
    time_it!(label_owned.as_str(), {
        download_with_concurrency_limit(
            tasks,
            #[cfg(feature = "events")]
            event_bus,
        )
        .await?
    });
    lighty_core::trace_info!("[Installer] {} installed", label);
    Ok(())
}
