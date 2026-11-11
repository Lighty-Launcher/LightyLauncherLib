// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Config file (`config/**`) installation, served by LightyUpdater.

use lighty_loaders::types::{version_metadata::Mods, VersionInfo};

use crate::errors::InstallerResult;
use crate::installer::downloader::DownloadTask;

#[cfg(feature = "events")]
use lighty_event::{Event, EventBus, ModloaderEvent};

use super::asset_partition;

pub async fn collect_config_tasks<'a>(
    version: &impl VersionInfo,
    mods: &'a [Mods],
) -> Vec<DownloadTask<'a>> {
    asset_partition::collect(version, mods, "config", false).await
}

pub async fn download_config(
    tasks: Vec<DownloadTask<'_>>,
    #[cfg(feature = "events")] event_bus: Option<&EventBus>,
) -> InstallerResult<()> {
    #[cfg(feature = "events")]
    let count = tasks.len();
    #[cfg(feature = "events")]
    let bytes: u64 = tasks.iter().map(|task| task.size).sum();
    asset_partition::download(
        tasks,
        "config",
        #[cfg(feature = "events")]
        event_bus,
    )
    .await?;

    #[cfg(feature = "events")]
    if count > 0 && let Some(bus) = event_bus {
        bus.emit(Event::Modloader(ModloaderEvent::ConfigInstalled {
            count,
            bytes,
        }));
    }
    Ok(())
}
