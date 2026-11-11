// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! File verification and cache checking utilities.

use lighty_core::verify_file_sha1_streaming;
use std::path::PathBuf;
use tokio::fs;

/// SHA1 of zero bytes: what a server declares for an intentionally empty file.
const EMPTY_FILE_SHA1: &str = "da39a3ee5e6b4b0d3255bfef95601890afd80709";

/// Returns whether the file at `path` is missing or empty.
///
/// The object store is content-addressed — the file name is the expected
/// SHA1 — so a file sitting there was already verified when written.
pub async fn is_missing_or_empty(path: &PathBuf) -> bool {
    match fs::metadata(path).await {
        Ok(meta) if meta.len() > 0 => false,
        Ok(_) => {
            lighty_core::trace_warn!(
                "[Installer] Zero-byte cached file at {}, re-downloading...",
                path.display()
            );
            let _ = fs::remove_file(path).await;
            true
        }
        Err(_) => true,
    }
}

/// Returns whether the file at `path` needs to be (re-)downloaded.
///
/// `true` when missing, zero-byte, or the SHA1 doesn't match.
pub async fn needs_download(path: &PathBuf, sha1: Option<&String>, name: &str) -> bool {
    if !path.exists() {
        return true;
    }

    // Zero-byte files are stale artifacts of failed downloads, unless the
    // server declares the file empty on purpose (some mods ship empty configs).
    if let Ok(meta) = fs::metadata(path).await {
        if meta.len() == 0 {
            if sha1.is_some_and(|hash| hash.eq_ignore_ascii_case(EMPTY_FILE_SHA1)) {
                return false;
            }

            lighty_core::trace_warn!(
                "[Installer] Zero-byte cached file for {}, re-downloading...",
                name
            );
            let _ = fs::remove_file(path).await;
            return true;
        }
    }

    if let Some(hash) = sha1 {
        match verify_file_sha1_streaming(path, hash).await {
            Ok(true) => false,
            _ => {
                lighty_core::trace_warn!(
                    "[Installer] SHA1 mismatch for {}, re-downloading...",
                    name
                );
                let _ = fs::remove_file(path).await;
                true
            }
        }
    } else {
        false
    }
}
