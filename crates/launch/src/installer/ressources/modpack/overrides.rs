// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Shared override-copy helper for modpack pipelines.
//!
//! Policy: SkipWarn — existing user files (options.txt, configs, keybinds)
//! are kept; the override is skipped with a `trace_warn!`.

use std::path::Path;

use lighty_core::QueryError;

use crate::errors::{InstallerError, InstallerResult};

/// Recursively copies `from` into `into`, keeping existing files.
/// Returns the count of files actually copied.
pub(crate) async fn extract_overrides(from: &Path, into: &Path) -> InstallerResult<usize> {
    let from = from.to_path_buf();
    let into = into.to_path_buf();
    tokio::task::spawn_blocking(move || copy_dir_skip(&from, &into))
        .await
        .map_err(|e| InstallerError::Query(QueryError::Conversion {
            message: format!("Overrides copy task panicked: {e}"),
        }))?
}

fn copy_dir_skip(from: &Path, into: &Path) -> InstallerResult<usize> {
    if !from.exists() {
        return Ok(0);
    }
    let mut count = 0;
    std::fs::create_dir_all(into)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let src = entry.path();
        let rel = src
            .strip_prefix(from)
            .map_err(|e| InstallerError::Query(QueryError::Conversion {
                message: format!("Override path normalization failed: {e}"),
            }))?;
        let dst = into.join(rel);
        if entry.file_type()?.is_dir() {
            count += copy_dir_skip(&src, &dst)?;
        } else {
            if dst.exists() {
                lighty_core::trace_warn!(
                    "[Modpack] Skipping override of existing file: {}",
                    dst.display()
                );
                continue;
            }
            if let Some(parent) = dst.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(&src, &dst)?;
            count += 1;
        }
    }
    Ok(count)
}
