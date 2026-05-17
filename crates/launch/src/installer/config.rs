// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Configuration for the installer module

use once_cell::sync::OnceCell;

/// Tuning knobs for the parallel file downloader.
#[derive(Debug, Clone, Copy)]
pub struct DownloaderConfig {
    pub max_concurrent_downloads: usize,
    pub max_retries: u32,
    pub initial_delay_ms: u64,
}

impl Default for DownloaderConfig {
    fn default() -> Self {
        Self {
            max_concurrent_downloads: 50,
            max_retries: 3,
            initial_delay_ms: 20,
        }
    }
}

/// Global downloader configuration; populated once on startup.
static DOWNLOADER_CONFIG: OnceCell<DownloaderConfig> = OnceCell::new();

/// Installs the downloader configuration. Call this once at startup.
///
/// # Example
///
/// ```no_run
/// use lighty_launch::installer::config::{DownloaderConfig, init_downloader_config};
///
/// init_downloader_config(DownloaderConfig {
///     max_concurrent_downloads: 100,
///     max_retries: 5,
///     initial_delay_ms: 50,
/// });
/// ```
pub fn init_downloader_config(config: DownloaderConfig) {
    DOWNLOADER_CONFIG.set(config).ok();
}

/// Returns the active downloader configuration (defaults if uninitialized).
pub(crate) fn get_config() -> DownloaderConfig {
    *DOWNLOADER_CONFIG.get_or_init(DownloaderConfig::default)
}
