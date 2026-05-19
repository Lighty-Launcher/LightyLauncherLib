// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! HTTP download helpers built on top of the shared `HTTP_CLIENT`.

use std::path::Path;

use tokio::io::AsyncWriteExt;

use crate::errors::DownloadResult;
use crate::hosts::HTTP_CLIENT;
use crate::trace_debug;

/// Streams `url` to `path` chunk by chunk. No progress callback —
/// reserved for small artefacts (modpack archives, single mod files).
pub async fn download_file_untracked(url: &str, path: impl AsRef<Path>) -> DownloadResult<()> {
    let mut response = HTTP_CLIENT.get(url).send().await?.error_for_status()?;
    let mut file = tokio::fs::File::create(path.as_ref()).await?;

    while let Some(chunk) = response.chunk().await? {
        file.write_all(&chunk).await?;
    }
    file.flush().await?;
    Ok(())
}

/// Downloads `url` into an in-memory buffer and reports progress after
/// every chunk via `on_progress(downloaded_bytes, total_bytes)`. The
/// total is `0` when the server doesn't expose `Content-Length`.
pub async fn download_file<F>(url: &str, on_progress: F) -> DownloadResult<Vec<u8>>
where
    F: Fn(u64, u64),
{
    let trimmed = url.trim();
    trace_debug!("Downloading {trimmed}");

    let mut response = HTTP_CLIENT.get(trimmed).send().await?.error_for_status()?;
    let total = response.content_length().unwrap_or(0);

    on_progress(0, total);

    let mut buffer = Vec::with_capacity(total as usize);
    let mut downloaded: u64 = 0;
    while let Some(chunk) = response.chunk().await? {
        buffer.extend_from_slice(&chunk);
        downloaded += chunk.len() as u64;
        on_progress(downloaded, total);
    }

    trace_debug!("Downloaded {} ({downloaded} bytes)", trimmed);
    Ok(buffer)
}
