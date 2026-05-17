// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Resource-pack (`resourcepacks/*.zip`) installation.

use std::path::PathBuf;

use lighty_loaders::types::{version_metadata::Mods, VersionInfo};

use crate::errors::InstallerResult;

#[cfg(feature = "events")]
use lighty_event::{Event, EventBus, ModloaderEvent};

use super::asset_partition;

pub async fn collect_resourcepack_tasks(
    version: &impl VersionInfo,
    mods: &[Mods],
) -> (Vec<(String, PathBuf)>, u64) {
    asset_partition::collect(version, mods, "resourcepacks", false).await
}

pub async fn download_resourcepacks(
    tasks: Vec<(String, PathBuf)>,
    bytes: u64,
    #[cfg(feature = "events")] event_bus: Option<&EventBus>,
) -> InstallerResult<()> {
    let count = tasks.len();
    asset_partition::download(
        tasks,
        "resourcepacks",
        #[cfg(feature = "events")]
        event_bus,
    )
    .await?;

    #[cfg(feature = "events")]
    if count > 0 {
        if let Some(bus) = event_bus {
            bus.emit(Event::Modloader(ModloaderEvent::ResourcePacksInstalled {
                count,
                bytes,
            }));
        }
    }
    Ok(())
}
