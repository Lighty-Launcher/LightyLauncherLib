// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! File download utilities with retry logic and concurrency control

use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::Semaphore;
use futures::future::try_join_all;
use futures::StreamExt;
use lighty_core::hosts::HTTP_CLIENT as CLIENT;
use lighty_core::mkdir;
use crate::errors::InstallerResult;
use crate::errors::InstallerError;
use super::config::get_config;

#[cfg(feature = "events")]
use lighty_event::{EventBus, Event, LaunchEvent};

/// Exponential backoff with up-to-50% jitter to prevent thundering herd.
fn calculate_retry_delay(base_delay_ms: u64, attempt: u32) -> u64 {
    let exponential_delay = base_delay_ms * 2u64.pow(attempt - 1);
    let jitter = fastrand::u64(0..=exponential_delay / 2);
    exponential_delay + jitter
}

/// Downloads small files (loaded entirely in memory)
pub async fn download_small_file(
    url: String,
    dest: PathBuf,
    #[cfg(feature = "events")] event_bus: Option<&EventBus>,
) -> InstallerResult<()> {
    let config = get_config();
    let mut last_error = None;

    for attempt in 1..=config.max_retries {
        match download_small_file_once(
            &url,
            &dest,
            #[cfg(feature = "events")]
            event_bus,
        ).await {
            Ok(_) => return Ok(()),
            Err(e) => {
                if attempt < config.max_retries {
                    let delay = calculate_retry_delay(config.initial_delay_ms, attempt);
                    lighty_core::trace_warn!(
                        "[Retry {}/{}] Failed to download {}: {}. Retrying in {}ms...",
                        attempt, config.max_retries, url, e, delay
                    );
                    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                }
                last_error = Some(e);
            }
        }
    }

    Err(last_error.unwrap_or_else(|| {
        InstallerError::DownloadFailed(format!(
            "Download failed after {} retries without specific error details: {}",
            config.max_retries, url
        ))
    }))
}

async fn download_small_file_once(
    url: &str,
    dest: &PathBuf,
    #[cfg(feature = "events")] event_bus: Option<&EventBus>,
) -> InstallerResult<()> {
    let bytes = CLIENT.get(url).send().await?.bytes().await?;

    #[cfg(feature = "events")]
    if let Some(bus) = event_bus {
        bus.emit(Event::Launch(LaunchEvent::InstallProgress {
            bytes: bytes.len() as u64,
        }));
    }

    if let Some(parent) = dest.parent() {
        mkdir!(parent);
    }

    fs::write(dest, bytes).await?;
    Ok(())
}

/// Downloads large files with streaming (memory efficient)
pub async fn download_large_file(
    url: String,
    dest: PathBuf,
    #[cfg(feature = "events")] event_bus: Option<&EventBus>,
) -> InstallerResult<()> {
    let config = get_config();
    let mut last_error = None;

    for attempt in 1..=config.max_retries {
        match download_large_file_once(
            &url,
            &dest,
            #[cfg(feature = "events")]
            event_bus,
        )
        .await
        {
            Ok(_) => return Ok(()),
            Err(e) => {
                if attempt < config.max_retries {
                    let delay = calculate_retry_delay(config.initial_delay_ms, attempt);
                    lighty_core::trace_warn!(
                        "[Retry {}/{}] Failed to download {}: {}. Retrying in {}ms...",
                        attempt, config.max_retries, url, e, delay
                    );
                    let _ = fs::remove_file(&dest).await;
                    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                }
                last_error = Some(e);
            }
        }
    }

    Err(last_error.unwrap_or_else(|| {
        InstallerError::DownloadFailed(format!(
            "Download failed after {} retries without specific error details: {}",
            config.max_retries, url
        ))
    }))
}

async fn download_large_file_once(
    url: &str,
    dest: &PathBuf,
    #[cfg(feature = "events")] event_bus: Option<&EventBus>,
) -> InstallerResult<()> {
    let response = CLIENT.get(url).send().await?;

    if !response.status().is_success() {
        return Err(InstallerError::DownloadFailed(format!(
            "HTTP {} for {}",
            response.status(),
            url
        )));
    }

    if let Some(parent) = dest.parent() {
        mkdir!(parent);
    }

    let file = fs::File::create(dest).await?;
    let mut writer = BufWriter::with_capacity(256 * 1024, file);
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        writer.write_all(&chunk).await?;

        #[cfg(feature = "events")]
        if let Some(bus) = event_bus {
            bus.emit(Event::Launch(LaunchEvent::InstallProgress {
                bytes: chunk.len() as u64,
            }));
        }
    }

    writer.flush().await?;
    Ok(())
}

/// Downloads multiple large files with concurrency limit
pub async fn download_with_concurrency_limit(
    tasks: Vec<(String, PathBuf)>,
    #[cfg(feature = "events")] event_bus: Option<&EventBus>,
) -> InstallerResult<()> {
    let config = get_config();
    let semaphore = Arc::new(Semaphore::new(config.max_concurrent_downloads));
    let futures: Vec<_> = tasks
        .into_iter()
        .map(|(url, dest)| {
            let sem = semaphore.clone();
            async move {
                let _permit = sem.acquire().await
                    .map_err(|_| InstallerError::DownloadFailed(
                        "Download concurrency semaphore closed".into()
                    ))?;
                download_large_file(
                    url,
                    dest,
                    #[cfg(feature = "events")]
                    event_bus,
                )
                .await
            }
        })
        .collect();

    try_join_all(futures).await?;
    Ok(())
}

/// Downloads multiple small files with concurrency limit
pub async fn download_small_with_concurrency_limit(
    tasks: Vec<(String, PathBuf)>,
    #[cfg(feature = "events")] event_bus: Option<&EventBus>,
) -> InstallerResult<()> {
    let config = get_config();
    let semaphore = Arc::new(Semaphore::new(config.max_concurrent_downloads));
    let futures: Vec<_> = tasks
        .into_iter()
        .map(|(url, dest)| {
            let sem = semaphore.clone();
            async move {
                let _permit = sem.acquire().await
                    .map_err(|_| InstallerError::DownloadFailed(
                        "Download concurrency semaphore closed".into()
                    ))?;
                download_small_file(
                    url,
                    dest,
                    #[cfg(feature = "events")]
                    event_bus,
                )
                .await
            }
        })
        .collect();

    try_join_all(futures).await?;
    Ok(())
}
