// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Shader-pack (`shaderpacks/*.zip`) installation.

use lighty_loaders::types::{version_metadata::Mods, VersionInfo};

use crate::errors::InstallerResult;
use crate::installer::downloader::DownloadTask;

#[cfg(feature = "events")]
use lighty_event::{Event, EventBus, ModloaderEvent};

use super::asset_partition;

pub async fn collect_shaderpack_tasks<'a>(
    version: &impl VersionInfo,
    mods: &'a [Mods],
) -> Vec<DownloadTask<'a>> {
    asset_partition::collect(version, mods, "shaderpacks", false).await
}

pub async fn download_shaderpacks(
    tasks: Vec<DownloadTask<'_>>,
    #[cfg(feature = "events")] event_bus: Option<&EventBus>,
) -> InstallerResult<()> {
    let count = tasks.len();
    #[cfg(feature = "events")]
    let bytes: u64 = tasks.iter().map(|task| task.size).sum();
    asset_partition::download(
        tasks,
        "shaderpacks",
        #[cfg(feature = "events")]
        event_bus,
    )
    .await?;

    #[cfg(feature = "events")]
    if count > 0 {
        if let Some(bus) = event_bus {
            bus.emit(Event::Modloader(ModloaderEvent::ShaderPacksInstalled {
                count,
                bytes,
            }));
        }
    }
    Ok(())
}
