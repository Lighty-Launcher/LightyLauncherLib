// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! OS-keychain handle for auth tokens (opt-in via the `keyring` feature).
//!
//! The token is stored encrypted by the OS (Keychain on macOS, Credential
//! Manager on Windows, Secret Service on Linux). The token only re-enters
//! process memory on explicit [`TokenHandle::read`] — typically right
//! before injecting it into the Minecraft `--accessToken` argv.

use std::fmt;

use ::keyring::Entry;
use secrecy::{ExposeSecret, SecretString};

use crate::AuthError;

/// Reference to a token stored in the OS keychain.
///
/// Created internally by [`MicrosoftAuth::with_keyring`](crate::MicrosoftAuth)
/// or [`AzuriomAuth::with_keyring`](crate::AzuriomAuth); not constructible
/// directly to enforce the "tokens are written by the provider" contract.
pub struct TokenHandle {
    service: String,
    username: String,
}

impl Clone for TokenHandle {
    fn clone(&self) -> Self {
        Self {
            service: self.service.clone(),
            username: self.username.clone(),
        }
    }
}

impl fmt::Debug for TokenHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TokenHandle")
            .field("service", &self.service)
            .field("username", &self.username)
            .finish()
    }
}

impl TokenHandle {
    pub(crate) fn new(service: impl Into<String>, username: impl Into<String>) -> Self {
        Self {
            service: service.into(),
            username: username.into(),
        }
    }

    fn entry(&self) -> Result<Entry, AuthError> {
        Ok(Entry::new(&self.service, &self.username)?)
    }

    pub(crate) fn store(&self, secret: &SecretString) -> Result<(), AuthError> {
        self.entry()?.set_password(secret.expose_secret())?;
        Ok(())
    }

    /// Reads the token from the OS keychain. The returned `SecretString`
    /// should be consumed immediately (passed to argv, wiped on drop).
    pub fn read(&self) -> Result<SecretString, AuthError> {
        Ok(SecretString::from(self.entry()?.get_password()?))
    }

    /// Deletes the entry from the OS keychain. Idempotent: a missing
    /// entry is treated as success.
    pub fn revoke(&self) -> Result<(), AuthError> {
        match self.entry()?.delete_credential() {
            Ok(()) => Ok(()),
            Err(::keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}
