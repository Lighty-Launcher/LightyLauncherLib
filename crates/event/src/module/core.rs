// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Core extraction events.

use serde::{Deserialize, Serialize};

/// Core extraction events.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum CoreEvent {
    ExtractionStarted {
        archive_type: String,
        file_count: usize,
        destination: String,
    },
    ExtractionProgress {
        files_extracted: usize,
        total_files: usize,
    },
    ExtractionCompleted {
        archive_type: String,
        files_extracted: usize,
    },
}
