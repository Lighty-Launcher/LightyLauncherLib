// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! SHA1 file hashing utilities (sync and async).

use std::path::Path;
use sha1::{Sha1, Digest};
use std::io::Read;
use tokio::fs;
use thiserror::Error;

/// Errors raised by the hashing helpers.
#[derive(Debug, Error)]
pub enum HashError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("SHA1 mismatch: expected {expected}, got {actual}")]
    Mismatch { expected: String, actual: String },
}

pub type HashResult<T> = Result<T, HashError>;

/// Verifies the SHA1 hash of a file (async, reads whole file into memory).
pub async fn verify_file_sha1(path: &Path, expected_sha1: &str) -> HashResult<bool> {
    let content = fs::read(path).await?;

    let mut hasher = Sha1::new();
    hasher.update(&content);
    let calculated_sha1 = hex::encode(hasher.finalize());

    Ok(calculated_sha1.eq_ignore_ascii_case(expected_sha1))
}

/// Verifies the SHA1 hash of a file (async, streaming for large files).
pub async fn verify_file_sha1_streaming(path: &Path, expected_sha1: &str) -> HashResult<bool> {
    use tokio::io::AsyncReadExt;

    let mut file = fs::File::open(path).await?;
    let mut hasher = Sha1::new();
    let mut buffer = [0u8; 8192];

    loop {
        let n = file.read(&mut buffer).await?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    let calculated_sha1 = hex::encode(hasher.finalize());
    Ok(calculated_sha1.eq_ignore_ascii_case(expected_sha1))
}

/// Calculates the SHA1 hash of a file (sync, blocking I/O — for non-async contexts).
pub fn calculate_file_sha1_sync(path: &Path) -> HashResult<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha1::new();
    let mut buffer = [0u8; 8192];

    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    Ok(hex::encode(hasher.finalize()))
}

/// Verifies the SHA1 hash of a file (sync).
pub fn verify_file_sha1_sync(path: &Path, expected_sha1: &str) -> HashResult<bool> {
    let calculated_sha1 = calculate_file_sha1_sync(path)?;
    Ok(calculated_sha1.eq_ignore_ascii_case(expected_sha1))
}

/// Calculates the SHA1 hash of arbitrary bytes as a lowercase hex string.
pub fn calculate_sha1_bytes(data: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Calculates the SHA1 hash of arbitrary bytes as raw 20-byte output.
pub fn calculate_sha1_bytes_raw(data: &[u8]) -> [u8; 20] {
    let mut hasher = Sha1::new();
    hasher.update(data);
    hasher.finalize().into()
}
