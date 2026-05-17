// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Shared CurseForge Core-API base URL, key storage, and URL helpers.

use std::sync::{PoisonError, RwLock};

use lighty_core::QueryError;
use once_cell::sync::Lazy;

pub const BASE_URL: &str = "https://api.curseforge.com/v1";

pub const PROVIDER: &str = "curseforge";

/// Process-wide API key, set before the first authenticated request.
static API_KEY: Lazy<RwLock<Option<String>>> = Lazy::new(|| RwLock::new(None));

/// Configures the CurseForge API key used by every subsequent request.
pub fn set_api_key(key: impl Into<String>) {
    let mut guard = API_KEY
        .write()
        .unwrap_or_else(PoisonError::into_inner);
    *guard = Some(key.into());
}

/// Returns the configured key or a clear error pointing at [`set_api_key`].
pub fn read_api_key() -> Result<String, QueryError> {
    let guard = API_KEY
        .read()
        .unwrap_or_else(PoisonError::into_inner);
    guard.clone().ok_or_else(|| QueryError::Conversion {
        message: "CurseForge API key not configured. Call \
                  `lighty_modsloader::curseforge::set_api_key(...)` \
                  before launching an instance with `.with_curseforge(...)`"
            .to_string(),
    })
}

/// Minimal RFC3986 URL-encoder.
pub fn url_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 8);
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => out.push_str(&format!("%{:02X}", byte)),
        }
    }
    out
}
