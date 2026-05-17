// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Authentication providers for Lighty Launcher (Offline, Microsoft, Azuriom).

mod auth;
mod errors;

#[cfg(feature = "keyring")]
pub mod keyring;

pub mod offline;
pub mod azuriom;
pub mod microsoft;

pub use auth::{
    AuthProvider, AuthResult, Authenticator, UserProfile, UserRole, generate_offline_uuid,
};
pub use errors::AuthError;
pub use secrecy::{ExposeSecret, SecretString};

#[cfg(feature = "keyring")]
pub use keyring::TokenHandle;
