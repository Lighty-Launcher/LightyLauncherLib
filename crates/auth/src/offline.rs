// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Offline authentication: deterministic UUID v5 derived from the username.

use crate::{Authenticator, AuthError, AuthProvider, AuthResult, UserProfile, generate_offline_uuid};

#[cfg(feature = "events")]
use lighty_event::{EventBus, Event, AuthEvent};

/// Offline authenticator — no network calls, suitable for offline play or testing.
pub struct OfflineAuth {
    username: String,
}

impl OfflineAuth {
    /// Create a new offline authenticator.
    pub fn new(username: impl Into<String>) -> Self {
        Self {
            username: username.into(),
        }
    }

    /// Get the username.
    pub fn username(&self) -> &str {
        &self.username
    }
}

impl Authenticator for OfflineAuth {
    async fn authenticate(
        &mut self,
        #[cfg(feature = "events")] event_bus: Option<&EventBus>,
    ) -> AuthResult<UserProfile> {
        #[cfg(feature = "events")]
        if let Some(bus) = event_bus {
            bus.emit(Event::Auth(AuthEvent::AuthenticationStarted {
                provider: "Offline".to_string(),
            }));
        }

        if self.username.is_empty() {
            #[cfg(feature = "events")]
            if let Some(bus) = event_bus {
                bus.emit(Event::Auth(AuthEvent::AuthenticationFailed {
                    provider: "Offline".to_string(),
                    error: "Username cannot be empty".to_string(),
                }));
            }
            return Err(AuthError::InvalidCredentials);
        }

        if self.username.len() < 3 || self.username.len() > 16 {
            let error_msg = "Username must be between 3 and 16 characters".to_string();
            #[cfg(feature = "events")]
            if let Some(bus) = event_bus {
                bus.emit(Event::Auth(AuthEvent::AuthenticationFailed {
                    provider: "Offline".to_string(),
                    error: error_msg.clone(),
                }));
            }
            return Err(AuthError::Custom(error_msg));
        }

        if !self.username.chars().all(|c| c.is_alphanumeric() || c == '_') {
            let error_msg = "Username can only contain letters, numbers, and underscores".to_string();
            #[cfg(feature = "events")]
            if let Some(bus) = event_bus {
                bus.emit(Event::Auth(AuthEvent::AuthenticationFailed {
                    provider: "Offline".to_string(),
                    error: error_msg.clone(),
                }));
            }
            return Err(AuthError::Custom(error_msg));
        }

        let uuid = generate_offline_uuid(&self.username);

        #[cfg(feature = "events")]
        if let Some(bus) = event_bus {
            bus.emit(Event::Auth(AuthEvent::AuthenticationSuccess {
                provider: "Offline".to_string(),
                username: self.username.clone(),
                uuid: uuid.clone(),
            }));
        }

        Ok(UserProfile {
            id: None,
            username: self.username.clone(),
            uuid,
            access_token: None,
            #[cfg(feature = "keyring")]
            token_handle: None,
            xuid: None,
            email: None,
            email_verified: false,
            money: None,
            role: None,
            banned: false,
            provider: AuthProvider::Offline,
        })
    }
}
