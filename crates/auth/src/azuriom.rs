// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Azuriom CMS authentication (email/password, 2FA, token verification, logout).

use crate::auth::route_token;
use crate::{Authenticator, AuthError, AuthProvider, AuthResult, UserProfile, UserRole};
use lighty_core::hosts::HTTP_CLIENT as CLIENT;
use serde::Deserialize;

#[cfg(feature = "events")]
use lighty_event::{EventBus, Event, AuthEvent};

/// Authenticates users via an Azuriom CMS instance.
pub struct AzuriomAuth {
    base_url: String,
    email: String,
    password: String,
    two_factor_code: Option<String>,
    #[cfg(feature = "keyring")]
    keyring_service: Option<String>,
}

impl AzuriomAuth {
    /// Create a new Azuriom authenticator.
    pub fn new(base_url: impl Into<String>, email: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            email: email.into(),
            password: password.into(),
            two_factor_code: None,
            #[cfg(feature = "keyring")]
            keyring_service: None,
        }
    }

    /// Set the 2FA code (call when authentication returned `TwoFactorRequired`).
    pub fn set_two_factor_code(&mut self, code: impl Into<String>) {
        self.two_factor_code = Some(code.into());
    }

    /// Clear the 2FA code.
    pub fn clear_two_factor_code(&mut self) {
        self.two_factor_code = None;
    }

    /// Route subsequent `access_token`s into the OS keychain under
    /// `service` (and `username = format!("azuriom:{uuid}")`). The
    /// returned `UserProfile` carries a [`TokenHandle`] instead of the
    /// raw token, so the secret never lives long-term in process memory.
    #[cfg(feature = "keyring")]
    pub fn with_keyring(mut self, service: impl Into<String>) -> Self {
        self.keyring_service = Some(service.into());
        self
    }

    fn keyring_service(&self) -> Option<&str> {
        #[cfg(feature = "keyring")]
        {
            self.keyring_service.as_deref()
        }
        #[cfg(not(feature = "keyring"))]
        {
            None
        }
    }
}


/// Azuriom API response for successful authentication.
#[derive(Debug, Deserialize)]
struct AzuriomAuthResponse {
    id: u64,
    username: String,
    uuid: String,
    access_token: String,
    email_verified: Option<bool>,
    money: Option<f64>,
    role: Option<AzuriomRole>,
    banned: Option<bool>,
}

/// Azuriom role information.
#[derive(Debug, Deserialize)]
struct AzuriomRole {
    name: String,
    color: Option<String>,
}

/// Azuriom API error response.
#[derive(Debug, Deserialize)]
struct AzuriomErrorResponse {
    status: String,
    reason: String,
    message: String,
}
impl Authenticator for AzuriomAuth {
    async fn authenticate(
        &mut self,
        #[cfg(feature = "events")] event_bus: Option<&EventBus>,
    ) -> AuthResult<UserProfile> {
        let url = format!("{}/api/auth/authenticate", self.base_url);
        lighty_core::trace_debug!(url = %url, email = %self.email, "Authenticating with Azuriom");

        #[cfg(feature = "events")]
        if let Some(bus) = event_bus {
            bus.emit(Event::Auth(AuthEvent::AuthenticationStarted {
                provider: "Azuriom".to_string(),
            }));
        }

        let mut body = serde_json::json!({
            "email": self.email,
            "password": self.password,
        });

        if let Some(code) = &self.two_factor_code {
            body["code"] = serde_json::json!(code);
        }

        let response = CLIENT
            .post(&url)
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        let response_text = response.text().await?;

        if status.is_success() {
            let azuriom_response: AzuriomAuthResponse = serde_json::from_str(&response_text)
                .map_err(|e| AuthError::InvalidResponse(format!("Failed to parse response: {}", e)))?;

            if azuriom_response.banned.unwrap_or(false) {
                lighty_core::trace_error!(username = %azuriom_response.username, "Account is banned");
                #[cfg(feature = "events")]
                if let Some(bus) = event_bus {
                    bus.emit(Event::Auth(AuthEvent::AuthenticationFailed {
                        provider: "Azuriom".to_string(),
                        error: "Account is banned".to_string(),
                    }));
                }
                return Err(AuthError::AccountBanned(
                    azuriom_response.username.clone()
                ));
            }

            lighty_core::trace_info!(username = %azuriom_response.username, uuid = %azuriom_response.uuid, "Successfully authenticated");

            #[cfg(feature = "events")]
            if let Some(bus) = event_bus {
                bus.emit(Event::Auth(AuthEvent::AuthenticationSuccess {
                    provider: "Azuriom".to_string(),
                    username: azuriom_response.username.clone(),
                    uuid: azuriom_response.uuid.clone(),
                }));
            }

            let routed = route_token(
                azuriom_response.access_token,
                self.keyring_service(),
                &format!("azuriom:{}", azuriom_response.uuid),
            )?;
            Ok(UserProfile {
                id: Some(azuriom_response.id),
                username: azuriom_response.username,
                uuid: azuriom_response.uuid,
                access_token: routed.access_token,
                #[cfg(feature = "keyring")]
                token_handle: routed.token_handle,
                xuid: None,
                email: Some(self.email.clone()),
                email_verified: azuriom_response.email_verified.unwrap_or(true),
                money: azuriom_response.money,
                role: azuriom_response.role.map(|r| UserRole {
                    name: r.name,
                    color: r.color,
                }),
                banned: azuriom_response.banned.unwrap_or(false),
                provider: AuthProvider::Azuriom { base_url: self.base_url.clone() },
            })
        } else {
            let error_response: AzuriomErrorResponse = serde_json::from_str(&response_text)
                .map_err(|_| AuthError::InvalidResponse(format!("HTTP {}: {}", status, response_text)))?;

            if error_response.status != "error" {
                return Err(AuthError::InvalidResponse(format!(
                    "HTTP {}: expected status='error', got status='{}'",
                    status, error_response.status
                )));
            }

            lighty_core::trace_error!(reason = %error_response.reason, message = %error_response.message, "Authentication failed");

            let error = match error_response.reason.as_str() {
                "invalid_credentials" => AuthError::InvalidCredentials,
                "2fa" => AuthError::TwoFactorRequired,
                "invalid_2fa" => AuthError::Invalid2FACode,
                "email_not_verified" => AuthError::EmailNotVerified,
                "banned" => AuthError::AccountBanned(String::new()),
                _ => AuthError::Custom(error_response.message.clone()),
            };

            #[cfg(feature = "events")]
            if let Some(bus) = event_bus {
                bus.emit(Event::Auth(AuthEvent::AuthenticationFailed {
                    provider: "Azuriom".to_string(),
                    error: error_response.message,
                }));
            }

            Err(error)
        }
    }

    async fn verify(&self, token: &str) -> AuthResult<UserProfile> {
        let url = format!("{}/api/auth/verify", self.base_url);
        lighty_core::trace_debug!(url = %url, "Verifying token");

        let response = CLIENT
            .post(&url)
            .json(&serde_json::json!({
                "access_token": token
            }))
            .send()
            .await?;

        let status = response.status();
        let response_text = response.text().await?;

        if status.is_success() {
            let azuriom_response: AzuriomAuthResponse = serde_json::from_str(&response_text)
                .map_err(|e| AuthError::InvalidResponse(format!("Failed to parse response: {}", e)))?;

            lighty_core::trace_info!(username = %azuriom_response.username, "Token verified successfully");

            let routed = route_token(
                azuriom_response.access_token,
                self.keyring_service(),
                &format!("azuriom:{}", azuriom_response.uuid),
            )?;
            Ok(UserProfile {
                id: Some(azuriom_response.id),
                username: azuriom_response.username,
                uuid: azuriom_response.uuid,
                access_token: routed.access_token,
                #[cfg(feature = "keyring")]
                token_handle: routed.token_handle,
                xuid: None,
                email: None,
                email_verified: azuriom_response.email_verified.unwrap_or(true),
                money: azuriom_response.money,
                role: azuriom_response.role.map(|r| UserRole {
                    name: r.name,
                    color: r.color,
                }),
                banned: azuriom_response.banned.unwrap_or(false),
                provider: AuthProvider::Azuriom { base_url: self.base_url.clone() },
            })
        } else {
            lighty_core::trace_error!(status = %status, "Token verification failed");
            Err(AuthError::InvalidToken)
        }
    }

    async fn logout(&self, token: &str) -> AuthResult<()> {
        let url = format!("{}/api/auth/logout", self.base_url);
        lighty_core::trace_debug!(url = %url, "Logging out");

        let response = CLIENT
            .post(&url)
            .json(&serde_json::json!({
                "access_token": token
            }))
            .send()
            .await?;

        if response.status().is_success() {
            lighty_core::trace_info!("Successfully logged out");
            Ok(())
        } else {
            lighty_core::trace_error!(status = %response.status(), "Logout failed");
            Err(AuthError::InvalidToken)
        }
    }
}


