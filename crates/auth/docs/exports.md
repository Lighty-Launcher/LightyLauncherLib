# Exports

Public surface of the `lighty-auth` crate.

## Module layout

```
lighty_auth
├── auth                         (private internals)
│   re-exported as crate root:
│   ├── Authenticator (trait)
│   ├── UserProfile (+ ::offline constructor)
│   ├── UserRole
│   ├── AuthProvider
│   ├── AuthResult<T>
│   └── generate_offline_uuid
├── offline
│   └── OfflineAuth
├── microsoft
│   └── MicrosoftAuth     (+ with_keyring under "keyring")
├── azuriom
│   └── AzuriomAuth       (+ with_keyring under "keyring")
├── keyring               (feature "keyring")
│   └── TokenHandle
└── errors                (private internals)
    └── AuthError (re-exported as crate root)
```

## Crate root

```rust
use lighty_auth::{
    // Trait
    Authenticator,

    // Types
    UserProfile,
    UserRole,
    AuthProvider,
    AuthResult,

    // Secrets (re-exported from `secrecy`)
    SecretString,
    ExposeSecret,

    // Helper
    generate_offline_uuid,

    // Errors
    AuthError,
};
```

Provider types live in submodules:

```rust
use lighty_auth::{
    offline::OfflineAuth,
    microsoft::MicrosoftAuth,
    azuriom::AzuriomAuth,
};
```

## OS keychain (feature `keyring`)

```rust
# #[cfg(feature = "keyring")]
use lighty_auth::TokenHandle;
```

`TokenHandle` is the opt-in pointer to a token stored in the OS
keychain. Not constructible directly — created by
`MicrosoftAuth::with_keyring(...)` and `AzuriomAuth::with_keyring(...)`.
Public methods:

| Method | Returns | Notes |
|---|---|---|
| `read()` | `AuthResult<SecretString>` | Fetches the token from the keychain |
| `revoke()` | `AuthResult<()>` | Deletes the entry (idempotent — `NoEntry` is treated as success) |

Enabling the feature also adds a variant
`AuthError::Keyring(keyring::Error)`.

## Type details

### `UserProfile`

```rust
pub struct UserProfile {
    pub id: Option<u64>,                     // Server-side user ID (Azuriom only)
    pub username: String,
    pub uuid: String,                        // dashed Minecraft UUID
    pub access_token: Option<SecretString>,  // secret-wrapped session / MC token
    #[cfg(feature = "keyring")]
    pub token_handle: Option<TokenHandle>,   // Opt-in OS-keychain handle
    pub xuid: Option<String>,                // Xbox User ID (Microsoft only)
    pub email: Option<String>,
    pub email_verified: bool,
    pub money: Option<f64>,
    pub role: Option<UserRole>,
    pub banned: bool,
    pub provider: AuthProvider,
}

impl UserProfile {
    pub fn offline(username: impl Into<String>, uuid: impl Into<String>) -> Self;
}
```

`UserProfile` is **not** `Serialize` / `Deserialize`. For "remember me"
persistence: enable `keyring` and call `with_keyring(...)`, or persist
only the MS refresh token yourself via the `keyring` crate. Pattern:
[microsoft.md → Silent re-auth](./microsoft.md#silent-re-auth).

### `UserRole`

```rust
pub struct UserRole {
    pub name: String,
    pub color: Option<String>,   // hex string, e.g. "#FFD700"
}
```

### `AuthProvider`

```rust
pub enum AuthProvider {
    Offline,
    Azuriom    { base_url: String },
    Microsoft  { client_id: String, refresh_token: Option<SecretString> },
    Custom     { base_url: String },
}
```

Also **not** `Serialize` / `Deserialize` (the secret-wrapped
`refresh_token` would defeat the purpose). The variant drives the
`${user_type}` launch placeholder: `Microsoft` → `"msa"`, `Azuriom` →
`"mojang"`, `Offline` / `Custom` → `"legacy"`.

### `AuthError`

```rust
pub enum AuthError {
    InvalidCredentials, TwoFactorRequired, Invalid2FACode,
    AccountBanned(String), EmailNotVerified,
    Network(reqwest::Error), InvalidResponse(String), InvalidToken,
    Cancelled, DeviceCodeExpired, Timeout,
    Serialization(serde_json::Error), Io(std::io::Error),
    #[cfg(feature = "keyring")] Keyring(keyring::Error),
    Custom(String),
}
```

### `Authenticator` trait

Full signature and implementation pattern: [trait.md](./trait.md).

## Cargo features

| Feature | Adds |
|---|---|
| `events` | `AuthEvent` emission through [`lighty-event`](../../event/docs/events.md) |
| `tracing` | `tracing` logs at the provider level |
| `keyring` | `TokenHandle`, `with_keyring(...)` on Microsoft / Azuriom, `AuthError::Keyring` |

`keyring` is forwarded from the umbrella crate as
`lighty-launcher/keyring` → `lighty-auth/keyring`
(`lighty-launch/keyring` also enables the matching path in the launch
crate for `--accessToken` injection).

## Related

- [Overview](./overview.md), [How to use](./how-to-use.md)
- [Trait](./trait.md) — custom authenticator skeleton
- [Events](./events.md) — `AuthEvent` lifecycle
