// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Datapack (`datapacks/*.zip`) installation. Only entries whose `path`
//! lives under a top-level `datapacks/` folder are picked up — per-world
//! datapacks (`saves/<world>/datapacks/...`) flow through the modpack
//! overrides path, not here.

use std::path::PathBuf;

use lighty_loaders::types::{version_metadata::Mods, VersionInfo};

use crate::errors::InstallerResult;

#[cfg(feature = "events")]
use lighty_event::{Event, EventBus, ModloaderEvent};

use super::asset_partition;

pub async fn collect_datapack_tasks(
    version: &impl VersionInfo,
    mods: &[Mods],
) -> (Vec<(String, PathBuf)>, u64) {
    asset_partition::collect(version, mods, "datapacks", false).await
}

pub async fn download_datapacks(
    tasks: Vec<(String, PathBuf)>,
    bytes: u64,
    #[cfg(feature = "events")] event_bus: Option<&EventBus>,
) -> InstallerResult<()> {
    let count = tasks.len();
    asset_partition::download(
        tasks,
        "datapacks",
        #[cfg(feature = "events")]
        event_bus,
    )
    .await?;

    #[cfg(feature = "events")]
    if count > 0 {
        if let Some(bus) = event_bus {
            bus.emit(Event::Modloader(ModloaderEvent::DatapacksInstalled {
                count,
                bytes,
            }));
        }
    }
    Ok(())
}
