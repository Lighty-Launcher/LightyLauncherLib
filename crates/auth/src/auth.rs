//! Core authentication types: [`Authenticator`] trait, [`UserProfile`],
//! [`UserRole`], [`AuthProvider`], plus [`generate_offline_uuid`].

use std::fmt;
use std::future::Future;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use crate::AuthError;

#[cfg(feature = "keyring")]
use crate::keyring::TokenHandle;

#[cfg(feature = "events")]
use lighty_event::EventBus;

pub type AuthResult<T> = Result<T, AuthError>;

/// User profile returned after successful authentication.
///
/// Intentionally not `Serialize` / `Deserialize`: dumping a profile in
/// plain JSON would leak the session token. See `AUTH_SECRETS.md`.
#[derive(Clone)]
pub struct UserProfile {
    pub id: Option<u64>,
    pub username: String,
    pub uuid: String,
    /// Minecraft/Azuriom session token. Wrapped in [`SecretString`] so
    /// `Debug` prints `[REDACTED]` and `serde` cannot serialise it.
    /// Read at launch-time via [`secrecy::ExposeSecret::expose_secret`].
    pub access_token: Option<SecretString>,
    /// Opt-in OS-keychain handle. When present, the token is stored
    /// outside the process address space and `access_token` is `None`.
    #[cfg(feature = "keyring")]
    pub token_handle: Option<TokenHandle>,
    pub xuid: Option<String>,
    pub email: Option<String>,
    pub email_verified: bool,
    pub money: Option<f64>,
    pub role: Option<UserRole>,
    pub banned: bool,
    pub provider: AuthProvider,
}

impl UserProfile {
    /// Minimal offline-mode profile — intended for tests, doctests and
    /// `OfflineAuth` integrations. All optional fields default to `None`.
    pub fn offline(username: impl Into<String>, uuid: impl Into<String>) -> Self {
        Self {
            id: None,
            username: username.into(),
            uuid: uuid.into(),
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
        }
    }
}

impl fmt::Debug for UserProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UserProfile")
            .field("id", &self.id)
            .field("username", &self.username)
            .field("uuid", &self.uuid)
            .field("access_token", &self.access_token.as_ref().map(|_| "[REDACTED]"))
            .field("xuid", &self.xuid)
            .field("email", &self.email)
            .field("email_verified", &self.email_verified)
            .field("money", &self.money)
            .field("role", &self.role)
            .field("banned", &self.banned)
            .field("provider", &self.provider)
            .finish()
    }
}

/// User role/rank information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRole {
    pub name: String,
    pub color: Option<String>,
}

/// Authentication provider type.
///
/// `Microsoft.refresh_token` is wrapped in [`SecretString`] for the
/// same reason as `UserProfile.access_token`. The enum is therefore
/// not `Serialize` / `Deserialize`.
#[derive(Debug, Clone)]
pub enum AuthProvider {
    Offline,
    Azuriom {
        base_url: String,
    },
    Microsoft {
        client_id: String,
        refresh_token: Option<SecretString>,
    },
    Custom {
        base_url: String,
    },
}

/// Helper return type for [`route_token`].
pub(crate) struct TokenRouting {
    pub access_token: Option<SecretString>,
    #[cfg(feature = "keyring")]
    pub token_handle: Option<TokenHandle>,
}

/// Wraps a freshly obtained token. If the provider was configured with
/// `with_keyring(service)`, the secret is written to the OS keychain
/// under `keyring_key` and only a [`TokenHandle`] is returned. Otherwise
/// the secret stays in process memory inside a [`SecretString`].
pub(crate) fn route_token(
    token: String,
    _keyring_service: Option<&str>,
    _keyring_key: &str,
) -> Result<TokenRouting, AuthError> {
    let secret = SecretString::from(token);
    #[cfg(feature = "keyring")]
    if let Some(service) = _keyring_service {
        let handle = TokenHandle::new(service, _keyring_key);
        handle.store(&secret)?;
        return Ok(TokenRouting {
            access_token: None,
            token_handle: Some(handle),
        });
    }
    Ok(TokenRouting {
        access_token: Some(secret),
        #[cfg(feature = "keyring")]
        token_handle: None,
    })
}

impl PartialEq for AuthProvider {
    fn eq(&self, other: &Self) -> bool {
        use secrecy::ExposeSecret;
        match (self, other) {
            (Self::Offline, Self::Offline) => true,
            (Self::Azuriom { base_url: a }, Self::Azuriom { base_url: b }) => a == b,
            (Self::Custom { base_url: a }, Self::Custom { base_url: b }) => a == b,
            (
                Self::Microsoft { client_id: ca, refresh_token: ta },
                Self::Microsoft { client_id: cb, refresh_token: tb },
            ) => {
                ca == cb
                    && ta.as_ref().map(|s| s.expose_secret().to_string())
                        == tb.as_ref().map(|s| s.expose_secret().to_string())
            }
            _ => false,
        }
    }
}

impl Eq for AuthProvider {}

/// Core authentication trait implemented by every provider.
pub trait Authenticator {
    /// Authenticate a user and return their profile.
    fn authenticate(
        &mut self,
        #[cfg(feature = "events")] event_bus: Option<&EventBus>,
    ) -> impl Future<Output = AuthResult<UserProfile>> + Send;

    /// Verify if a token is still valid.
    fn verify(&self, token: &str) -> impl Future<Output = AuthResult<UserProfile>> + Send {
        async move {
            let _ = token;
            Err(AuthError::Custom("Verification not supported for this provider".into()))
        }
    }

    /// Logout and invalidate the token.
    fn logout(&self, token: &str) -> impl Future<Output = AuthResult<()>> + Send {
        async move {
            let _ = token;
            Ok(())
        }
    }
}

/// Generates a deterministic UUID v5 (SHA1-based) from a username for offline mode.
pub fn generate_offline_uuid(username: &str) -> String {
    const NAMESPACE: &[u8] = b"OfflinePlayer:";

    let mut data = Vec::with_capacity(NAMESPACE.len() + username.len());
    data.extend_from_slice(NAMESPACE);
    data.extend_from_slice(username.as_bytes());

    let hash = lighty_core::calculate_sha1_bytes_raw(&data);

    // Version bits: 0101 (5) in the 13th position
    // Variant bits: 10xx in the 17th position (RFC 4122)
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-5{:01x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        hash[0], hash[1], hash[2], hash[3],
        hash[4], hash[5],
        hash[6] & 0x0f, hash[7],
        (hash[8] & 0x3f) | 0x80, hash[9],
        hash[10], hash[11], hash[12], hash[13], hash[14], hash[15]
    )
}
