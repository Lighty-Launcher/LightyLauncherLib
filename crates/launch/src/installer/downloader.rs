// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! File download utilities with retry logic and concurrency control

use std::path::{Path, PathBuf};
use std::sync::Arc;
use reqwest::header::RANGE;
use reqwest::StatusCode;
use sha1::{Digest, Sha1};
use tokio::fs;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::sync::Semaphore;
use futures::future::try_join_all;
use futures::StreamExt;
use lighty_core::calculate_sha1_bytes;
use lighty_core::hosts::HTTP_CLIENT as CLIENT;
use lighty_core::mkdir;
use crate::errors::InstallerResult;
use crate::errors::InstallerError;
use super::config::get_config;

#[cfg(feature = "events")]
use lighty_event::{EventBus, Event, LaunchEvent};

/// One file to fetch. `url` and `sha1` borrow the manifest that is alive for
/// the whole install, so building the list copies no strings; only `dest` is
/// owned, being a path that exists nowhere else.
pub struct DownloadTask<'a> {
    pub url: &'a str,
    pub dest: PathBuf,
    pub sha1: Option<&'a str>,
    pub size: u64,
}

/// Exponential backoff with up-to-50% jitter to prevent thundering herd.
fn calculate_retry_delay(base_delay_ms: u64, attempt: u32) -> u64 {
    let exponential_delay = base_delay_ms * 2u64.pow(attempt - 1);
    let jitter = fastrand::u64(0..=exponential_delay / 2);
    exponential_delay + jitter
}

/// Downloads small files (loaded entirely in memory)
pub async fn download_small_file(
    url: &str,
    dest: &Path,
    sha1: Option<&str>,
    #[cfg(feature = "events")] event_bus: Option<&EventBus>,
) -> InstallerResult<()> {
    let config = get_config();
    let mut last_error = None;

    for attempt in 1..=config.max_retries {
        match download_small_file_once(
            url,
            dest,
            sha1,
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

    Err(last_error.unwrap_or_else(|| InstallerError::RetriesExhausted {
        attempts: config.max_retries,
        url: url.to_string(),
    }))
}

async fn download_small_file_once(
    url: &str,
    dest: &Path,
    sha1: Option<&str>,
    #[cfg(feature = "events")] event_bus: Option<&EventBus>,
) -> InstallerResult<()> {
    let response = CLIENT.get(url).send().await?;

    if !response.status().is_success() {
        return Err(InstallerError::HttpStatus {
            status: response.status().as_u16(),
            url: url.to_string(),
        });
    }

    let bytes = response.bytes().await?;

    if let Some(expected) = sha1 {
        let digest = calculate_sha1_bytes(&bytes);

        if !digest.eq_ignore_ascii_case(expected) {
            return Err(InstallerError::Sha1Mismatch {
                url: url.to_string(),
                expected: expected.to_string(),
                actual: digest,
            });
        }
    }

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
    url: &str,
    dest: &Path,
    sha1: Option<&str>,
    #[cfg(feature = "events")] event_bus: Option<&EventBus>,
) -> InstallerResult<()> {
    let config = get_config();
    let mut last_error = None;

    for attempt in 1..=config.max_retries {
        match download_large_file_once(
            url,
            dest,
            sha1,
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
                    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                }
                last_error = Some(e);
            }
        }
    }

    Err(last_error.unwrap_or_else(|| InstallerError::RetriesExhausted {
        attempts: config.max_retries,
        url: url.to_string(),
    }))
}

async fn download_large_file_once(
    url: &str,
    dest: &Path,
    sha1: Option<&str>,
    #[cfg(feature = "events")] event_bus: Option<&EventBus>,
) -> InstallerResult<()> {
    // Resume what a previous attempt left behind: a dropped connection then
    // costs neither the bytes already received nor a second progress count.
    let downloaded = fs::metadata(dest).await.map(|meta| meta.len()).unwrap_or(0);

    let mut request = CLIENT.get(url);
    if downloaded > 0 {
        request = request.header(RANGE, format!("bytes={}-", downloaded));
    }

    let response = request.send().await?;
    let status = response.status();

    // The partial outgrew the file itself; drop it so the next attempt starts clean.
    if status == StatusCode::RANGE_NOT_SATISFIABLE {
        let _ = fs::remove_file(dest).await;
        return Err(InstallerError::StalePartialFile {
            url: url.to_string(),
        });
    }

    if !status.is_success() {
        return Err(InstallerError::HttpStatus {
            status: status.as_u16(),
            url: url.to_string(),
        });
    }

    if let Some(parent) = dest.parent() {
        mkdir!(parent);
    }

    let mut hasher = Sha1::new();

    // Only 206 keeps what is on disk; a 200 means the server ignored the
    // header and is sending the whole body again.
    let file = if status == StatusCode::PARTIAL_CONTENT {
        seed_hasher(dest, &mut hasher).await?;
        OpenOptions::new().append(true).open(dest).await?
    } else {
        fs::File::create(dest).await?
    };
    let mut writer = BufWriter::with_capacity(256 * 1024, file);
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        writer.write_all(&chunk).await?;
        hasher.update(&chunk);

        #[cfg(feature = "events")]
        if let Some(bus) = event_bus {
            bus.emit(Event::Launch(LaunchEvent::InstallProgress {
                bytes: chunk.len() as u64,
            }));
        }
    }

    writer.flush().await?;

    if let Some(expected) = sha1 {
        let digest = hex::encode(hasher.finalize());

        // Drop the file, or the retry would resume from the corrupt prefix
        // and fail every remaining attempt.
        if !digest.eq_ignore_ascii_case(expected) {
            let _ = fs::remove_file(dest).await;
            return Err(InstallerError::Sha1Mismatch {
                url: url.to_string(),
                expected: expected.to_string(),
                actual: digest,
            });
        }
    }

    Ok(())
}

/// Feeds the bytes already on disk into `hasher` so a resumed download still
/// ends up with the digest of the whole file.
async fn seed_hasher(dest: &Path, hasher: &mut Sha1) -> InstallerResult<()> {
    let mut partial = fs::File::open(dest).await?;
    let mut buffer = vec![0u8; 256 * 1024];

    loop {
        let read = partial.read(&mut buffer).await?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    Ok(())
}

/// Downloads multiple large files with concurrency limit
pub async fn download_with_concurrency_limit(
    tasks: Vec<DownloadTask<'_>>,
    #[cfg(feature = "events")] event_bus: Option<&EventBus>,
) -> InstallerResult<()> {
    let config = get_config();
    let semaphore = Arc::new(Semaphore::new(config.max_concurrent_downloads));
    let futures: Vec<_> = tasks
        .iter()
        .map(|task| {
            let sem = semaphore.clone();
            async move {
                let _permit = sem.acquire().await
                    .map_err(|_| InstallerError::ConcurrencyClosed)?;
                download_large_file(
                    task.url,
                    &task.dest,
                    task.sha1,
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
    tasks: Vec<DownloadTask<'_>>,
    #[cfg(feature = "events")] event_bus: Option<&EventBus>,
) -> InstallerResult<()> {
    let config = get_config();
    let semaphore = Arc::new(Semaphore::new(config.max_concurrent_downloads));
    let futures: Vec<_> = tasks
        .iter()
        .map(|task| {
            let sem = semaphore.clone();
            async move {
                let _permit = sem.acquire().await
                    .map_err(|_| InstallerError::ConcurrencyClosed)?;
                download_small_file(
                    task.url,
                    &task.dest,
                    task.sha1,
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
