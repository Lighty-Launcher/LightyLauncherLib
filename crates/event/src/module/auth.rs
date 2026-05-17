// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Authentication events.

use serde::{Deserialize, Serialize};

/// Authentication events.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum AuthEvent {
    AuthenticationStarted {
        provider: String,
    },
    AuthenticationInProgress {
        provider: String,
        step: String,
    },
    AuthenticationSuccess {
        provider: String,
        username: String,
        uuid: String,
    },
    AuthenticationFailed {
        provider: String,
        error: String,
    },
    AlreadyAuthenticated {
        provider: String,
        username: String,
    },
}
