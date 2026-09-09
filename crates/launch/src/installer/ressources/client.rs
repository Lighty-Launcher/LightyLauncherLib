// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Client JAR installation.

use lighty_loaders::types::{VersionInfo, version_metadata::Client};
use lighty_core::time_it;
use crate::errors::InstallerResult;
use crate::installer::verifier::needs_download;
use crate::installer::downloader::{download_large_file, DownloadTask};

#[cfg(feature = "events")]
use lighty_event::EventBus;

/// Collects the client JAR task if it needs downloading.
pub async fn collect_client_task<'a>(
    version: &impl VersionInfo,
    client: Option<&'a Client>,
) -> Option<DownloadTask<'a>> {
    let client = client?;
    let url = client.url.as_deref()?;
    let client_path = version.game_dirs().join(format!("{}.jar", version.name()));

    needs_download(&client_path, client.sha1.as_ref(), "Client JAR")
        .await
        .then(|| DownloadTask {
            url,
            dest: client_path,
            sha1: client.sha1.as_deref(),
            size: client.size.unwrap_or(0),
        })
}

/// Downloads client JAR from pre-collected task.
pub async fn download_client(
    task: Option<DownloadTask<'_>>,
    #[cfg(feature = "events")] event_bus: Option<&EventBus>,
) -> InstallerResult<()> {
    let Some(task) = task else {
        lighty_core::trace_info!("[Installer] Client JAR already cached and verified");
        return Ok(());
    };

    lighty_core::trace_info!("[Installer] Downloading client JAR...");
    time_it!(
        "Client download",
        download_large_file(
            task.url,
            &task.dest,
            task.sha1,
            #[cfg(feature = "events")]
            event_bus,
        )
        .await?
    );
    lighty_core::trace_info!("[Installer] Client JAR installed");
    Ok(())
}
