// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Java (JRE) events.

use serde::{Deserialize, Serialize};

/// Java (JRE) events.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum JavaEvent {
    JavaNotFound {
        distribution: String,
        version: u8,
    },
    JavaAlreadyInstalled {
        distribution: String,
        version: u8,
        binary_path: String,
    },
    JavaDownloadStarted {
        distribution: String,
        version: u8,
        total_bytes: u64,
    },
    JavaDownloadProgress {
        bytes: u64,
    },
    JavaDownloadCompleted {
        distribution: String,
        version: u8,
    },
    JavaExtractionStarted {
        distribution: String,
        version: u8,
    },
    JavaExtractionProgress {
        files_extracted: usize,
        total_files: usize,
    },
    JavaExtractionCompleted {
        distribution: String,
        version: u8,
        binary_path: String,
    },
}
