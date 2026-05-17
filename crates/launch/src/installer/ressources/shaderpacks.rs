// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Shader-pack (`shaderpacks/*.zip`) installation.

use std::path::PathBuf;

use lighty_loaders::types::{version_metadata::Mods, VersionInfo};

use crate::errors::InstallerResult;

#[cfg(feature = "events")]
use lighty_event::{Event, EventBus, ModloaderEvent};

use super::asset_partition;

pub async fn collect_shaderpack_tasks(
    version: &impl VersionInfo,
    mods: &[Mods],
) -> (Vec<(String, PathBuf)>, u64) {
    asset_partition::collect(version, mods, "shaderpacks", false).await
}

pub async fn download_shaderpacks(
    tasks: Vec<(String, PathBuf)>,
    bytes: u64,
    #[cfg(feature = "events")] event_bus: Option<&EventBus>,
) -> InstallerResult<()> {
    let count = tasks.len();
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
