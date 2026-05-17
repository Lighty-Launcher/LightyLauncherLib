# Exports

## Overview

Complete reference of all exports from `lighty-auth` and their re-exports in `lighty-launcher`.

## In `lighty_auth`

### Core Trait

```rust
use lighty_auth::Authenticator;
```

### Types

```rust
use lighty_auth::{
    UserProfile,     // Authenticated user data
    UserRole,        // User role/rank information
    AuthProvider,    // Provider type enum
    AuthResult,      // Result type alias: Result<T, AuthError>
};
```

### Secrets

Re-exported from the [`secrecy`](https://docs.rs/secrecy/) crate so
callers don't have to add it manually:

```rust
use lighty_auth::{SecretString, ExposeSecret};

// Read a token at launch time, right before injecting it into argv:
let token = profile.access_token
    .as_ref()
    .map(|s| s.expose_secret().to_owned());
```

### OS Keychain (feature `keyring`)

```rust
#[cfg(feature = "keyring")]
use lighty_auth::TokenHandle;
```

`TokenHandle` is the opt-in pointer to a token stored in the OS
keychain. Created internally by `MicrosoftAuth::with_keyring(...)` /
`AzuriomAuth::with_keyring(...)`; not constructible directly. Public
methods:

- `read() -> AuthResult<SecretString>` — fetches the token from the keychain
- `revoke() -> AuthResult<()>` — deletes the entry (idempotent)

### Helper Functions

```rust
use lighty_auth::generate_offline_uuid;
```

### Authentication Providers

```rust
use lighty_auth::{
    offline::OfflineAuth,
    microsoft::MicrosoftAuth,
    azuriom::AzuriomAuth,
};
```

### Errors

```rust
use lighty_auth::AuthError;
```

Adds variant `AuthError::Keyring(keyring::Error)` when the `keyring`
feature is enabled.

## In `lighty_launcher` (Re-exports)

```rust
use lighty_launcher::auth::{
    // Trait
    Authenticator,

    // Types
    UserProfile,
    UserRole,
    AuthProvider,
    AuthResult,

    // Secrets
    SecretString,
    ExposeSecret,

    // OS keychain (feature "keyring")
    #[cfg(feature = "keyring")]
    TokenHandle,

    // Helper
    generate_offline_uuid,

    // Providers
    offline::OfflineAuth,
    microsoft::MicrosoftAuth,
    azuriom::AzuriomAuth,

    // Errors
    AuthError,
};
```

## Usage Patterns

### Pattern 1: Direct Crate Import

```rust
use lighty_auth::{Authenticator, offline::OfflineAuth};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut auth = OfflineAuth::new("Player");

    #[cfg(not(feature = "events"))]
    let profile = auth.authenticate().await?;

    println!("{}", profile.username);
    Ok(())
}
```

### Pattern 2: Via Main Launcher Crate

```rust
use lighty_launcher::auth::{Authenticator, microsoft::MicrosoftAuth};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut auth = MicrosoftAuth::new("client-id");

    #[cfg(not(feature = "events"))]
    let profile = auth.authenticate().await?;

    Ok(())
}
```

## Type Details

### UserProfile

```rust
pub struct UserProfile {
    pub id: Option<u64>,                     // Server-side user ID (Azuriom only)
    pub username: String,
    pub uuid: String,                        // Minecraft UUID, with dashes
    pub access_token: Option<SecretString>,  // Session / MC token (secret-wrapped)
    #[cfg(feature = "keyring")]
    pub token_handle: Option<TokenHandle>,   // Opt-in OS-keychain handle
    pub xuid: Option<String>,                // Xbox User ID (Microsoft only)
    pub email: Option<String>,
    pub email_verified: bool,
    pub money: Option<f64>,
    pub role: Option<UserRole>,
    pub banned: bool,
    pub provider: AuthProvider,              // Which authenticator produced this profile
}

impl UserProfile {
    pub fn offline(username: impl Into<String>, uuid: impl Into<String>) -> Self;
}
```

`UserProfile` is **not** `Serialize` / `Deserialize` — dumping a profile
in plain JSON would leak the session token. For "remember me", either:

- enable the `keyring` feature and call
  `MicrosoftAuth::with_keyring(...)` / `AzuriomAuth::with_keyring(...)`
  to route the token into the OS keychain automatically; or
- persist only the MS `refresh_token` (a `SecretString`) yourself via
  the `keyring` crate's `Entry::set_password`.

See [`AUTH_SECRETS.md`](../../../AUTH_SECRETS.md) for the rationale.

### UserRole

```rust
pub struct UserRole {
    pub name: String,
    pub color: Option<String>,        // Hex format: #RRGGBB
}
```

### AuthProvider

```rust
pub enum AuthProvider {
    Offline,
    Azuriom {
        base_url: String,
    },
    Microsoft {
        client_id: String,
        /// MS refresh token (~90 days, rotates per RFC 6749).
        /// Wrapped in `SecretString` and consumed by
        /// `MicrosoftAuth::authenticate_with_refresh_token` to skip
        /// the device-code prompt on subsequent launches.
        refresh_token: Option<SecretString>,
    },
    Custom {
        base_url: String,
    },
}
```

`AuthProvider` is **not** `Serialize` / `Deserialize` (the secret-wrapped
`refresh_token` would defeat the purpose).

The variant drives the `${user_type}` launch placeholder at JVM start:
`Microsoft` → `"msa"`, `Azuriom` → `"mojang"`, `Offline`/`Custom` →
`"legacy"`.

### AuthError

```rust
pub enum AuthError {
    InvalidCredentials,
    TwoFactorRequired,
    Invalid2FACode,
    AccountBanned(String),
    EmailNotVerified,
    Network(reqwest::Error),
    InvalidResponse(String),
    InvalidToken,
    Cancelled,
    DeviceCodeExpired,
    Timeout,
    Serialization(serde_json::Error),
    Io(std::io::Error),
    #[cfg(feature = "keyring")]
    Keyring(keyring::Error),
    Custom(String),
}
```

## Module Structure

```
lighty_auth
├── auth
│   ├── Authenticator (trait)
│   ├── UserProfile (+ ::offline constructor)
│   ├── UserRole
│   ├── AuthProvider
│   ├── AuthResult<T>
│   └── generate_offline_uuid
├── offline
│   └── OfflineAuth
├── microsoft
│   └── MicrosoftAuth (+ with_keyring under "keyring")
├── azuriom
│   └── AzuriomAuth   (+ with_keyring under "keyring")
├── keyring          (feature "keyring")
│   └── TokenHandle
└── errors
    └── AuthError
```

## Cargo Features

| Feature   | Adds                                                        |
|-----------|-------------------------------------------------------------|
| `events`  | `AuthEvent` emission through `lighty-event`                 |
| `tracing` | `tracing` logs at the provider level                        |
| `keyring` | `TokenHandle`, `with_keyring(...)` on Microsoft / Azuriom, `AuthError::Keyring` |

`keyring` is forwarded from the root crate as
`lighty-launcher/keyring` → `lighty-auth/keyring`
(`lighty-launch/keyring` also enables the matching path in the launch
crate for `--accessToken` injection).

## Related Documentation

- [How to Use](./how-to-use.md) - Practical usage examples
- [Events](./events.md) - AuthEvent types
- [Trait](./trait.md) - Implementing custom authenticators
- [Overview](./overview.md) - Architecture overview
