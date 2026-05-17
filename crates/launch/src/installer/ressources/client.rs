// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Client JAR installation.

use lighty_loaders::types::{VersionInfo, version_metadata::Client};
use lighty_core::time_it;
use crate::errors::InstallerResult;
use crate::installer::verifier::needs_download;
use crate::installer::downloader::download_large_file;

#[cfg(feature = "events")]
use lighty_event::EventBus;

/// Collects client JAR task if it needs to be downloaded.
pub async fn collect_client_task(
    version: &impl VersionInfo,
    client: Option<&Client>,
) -> Option<(String, std::path::PathBuf)> {
    let client = client?;
    let url = client.url.as_ref()?.clone();
    let client_path = version.game_dirs().join(format!("{}.jar", version.name()));

    if needs_download(&client_path, client.sha1.as_ref(), "Client JAR").await {
        Some((url, client_path))
    } else {
        None
    }
}

/// Downloads client JAR from pre-collected task.
pub async fn download_client(
    task: Option<(String, std::path::PathBuf)>,
    #[cfg(feature = "events")] event_bus: Option<&EventBus>,
) -> InstallerResult<()> {
    let Some((url, client_path)) = task else {
        lighty_core::trace_info!("[Installer] Client JAR already cached and verified");
        return Ok(());
    };

    lighty_core::trace_info!("[Installer] Downloading client JAR...");
    time_it!(
        "Client download",
        download_large_file(
            url,
            client_path,
            #[cfg(feature = "events")]
            event_bus,
        )
        .await?
    );
    lighty_core::trace_info!("[Installer] Client JAR installed");
    Ok(())
}
