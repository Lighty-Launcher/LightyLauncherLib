// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Launch and installation events.

use serde::{Deserialize, Serialize};

/// Launch and installation events.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum LaunchEvent {
    IsInstalled {
        version: String,
    },
    InstallStarted {
        version: String,
        total_bytes: u64,
    },
    InstallProgress {
        bytes: u64,
    },
    InstallCompleted {
        version: String,
        total_bytes: u64,
    },
    Launching {
        version: String,
    },
    Launched {
        version: String,
        pid: u32,
    },
    NotLaunched {
        version: String,
        error: String,
    },
    ProcessOutput {
        pid: u32,
        stream: String,
        line: String,
    },
    ProcessExited {
        pid: u32,
        exit_code: i32,
    },
}
