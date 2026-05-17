# How to Use lighty-auth

## Basic Usage

### Offline Authentication

For local development and testing without network access:

```rust
use lighty_auth::{offline::OfflineAuth, Authenticator};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create offline authenticator
    let mut auth = OfflineAuth::new("Player123");

    // Authenticate (without events)
    #[cfg(not(feature = "events"))]
    let profile = auth.authenticate().await?;

    // Authenticate (with events)
    #[cfg(feature = "events")]
    let profile = auth.authenticate(None).await?;

    println!("Username: {}", profile.username);
    println!("UUID: {}", profile.uuid); // Deterministic UUID

    Ok(())
}
```

**Key features**:
- No network required
- Deterministic UUID generation (same username = same UUID)
- Perfect for development and testing

### Microsoft Authentication

OAuth 2.0 Device Code Flow for legitimate Minecraft accounts:

```rust
use lighty_auth::{microsoft::MicrosoftAuth, Authenticator};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize with Azure client ID
    let mut auth = MicrosoftAuth::new("your-azure-client-id");

    // Set callback for device code display
    auth.set_device_code_callback(|code, url| {
        println!("\n=== Microsoft Login ===");
        println!("Visit: {}", url);
        println!("Enter code: {}", code);
        println!("======================\n");
    });

    // Authenticate (without events)
    #[cfg(not(feature = "events"))]
    let profile = auth.authenticate().await?;

    // Authenticate (with events)
    #[cfg(feature = "events")]
    let profile = auth.authenticate(None).await?;

    println!("Logged in as: {}", profile.username);
    println!("UUID: {}", profile.uuid);
    // `access_token` is a `SecretString`. Expose it only at the exact
    // moment you need the bytes — Debug-printing the secret prints
    // `[REDACTED]` by design.
    if let Some(secret) = &profile.access_token {
        use secrecy::ExposeSecret;
        println!("Access token: {}", secret.expose_secret());
    }

    Ok(())
}
```

**Key features**:
- Device code flow (no embedded browser needed)
- Full Xbox Live and Minecraft Services integration
- Returns Minecraft access token for session validation (as a
  `SecretString`)
- Captures the MS refresh token in `profile.provider` for silent
  re-auth on subsequent launches (see below)
- Opt-in OS-keychain routing via `MicrosoftAuth::with_keyring("…")`
  (feature `keyring`) — the access token never stays in process
  memory long-term; only a [`TokenHandle`](./exports.md#os-keychain-feature-keyring)
  is returned on the profile

### Silent Re-authentication (Microsoft "Remember Me")

After the first device-code authentication, the resulting `UserProfile`
carries the MS refresh token under
`AuthProvider::Microsoft { refresh_token: Some(_), .. }`. Persist the
whole profile, then on the next launch call
`MicrosoftAuth::authenticate_with_refresh_token` instead of
`authenticate` — no device-code prompt, no user interaction.

```rust
use lighty_launcher::prelude::*;

let mut auth = MicrosoftAuth::new("your-azure-client-id");

// 1) Try silent first using the refresh token from a previous run.
let profile = match load_saved_profile() {
    Some(saved) => match saved.provider {
        AuthProvider::Microsoft { refresh_token: Some(rt), .. } => {
            auth.authenticate_with_refresh_token(&rt, None).await.ok()
        }
        _ => None,
    },
    None => None,
};

// 2) Fall back to device-code if no token saved or refresh failed
//    (token expired after ~90 days of inactivity, or was revoked).
let profile = match profile {
    Some(p) => p,
    None => {
        auth.set_device_code_callback(|code, url| {
            println!("Visit {url} and enter: {code}");
        });
        auth.authenticate(None).await?
    }
};

// 3) Persist again — Microsoft rotates the refresh token on every use
//    (RFC 6749), and the new one is now inside `profile.provider`.
save_profile(&profile)?;
```

**Recommended storage**: the OS-native secure store via the
[`keyring`](https://crates.io/crates/keyring) crate — Linux Secret
Service / macOS Keychain / Windows Credential Manager, encrypted at
rest by the OS. The library does not depend on `keyring`; persistence
is intentionally left to the consumer.

A complete runnable example with keyring lives at
[`examples/auth/microsoft.rs`](../../../examples/auth/microsoft.rs).

### Azuriom Authentication

Server-based authentication with custom CMS:

```rust
use lighty_auth::{azuriom::AzuriomAuth, Authenticator, AuthError};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize with server URL and credentials
    let mut auth = AzuriomAuth::new(
        "https://yourserver.com",
        "user@example.com",
        "password123"
    );

    // Authenticate (without events)
    #[cfg(not(feature = "events"))]
    match auth.authenticate().await {
        Ok(profile) => {
            println!("Logged in as: {}", profile.username);
            println!("Email: {}", profile.email.unwrap_or_default());
            println!("Money: {}", profile.money.unwrap_or(0.0));

            if let Some(role) = &profile.role {
                println!("Role: {} ({})", role.name, role.color.as_ref().unwrap_or(&"#FFFFFF".to_string()));
            }
        }
        Err(AuthError::TwoFactorRequired) => {
            println!("2FA required!");
            // Get code from user
            let code = "123456"; // Get from UI input
            auth.set_two_factor_code(code);

            // Retry authentication with 2FA code
            let profile = auth.authenticate().await?;
            println!("Logged in with 2FA: {}", profile.username);
        }
        Err(e) => {
            eprintln!("Authentication failed: {}", e);
        }
    }

    Ok(())
}
```

**Key features**:
- Two-factor authentication support
- User roles with colors
- Money/credits tracking
- Email verification status

## With Events

Track authentication progress with events:

```rust
use lighty_auth::{microsoft::MicrosoftAuth, Authenticator};
use lighty_event::{EventBus, Event, AuthEvent};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create event bus
    let event_bus = EventBus::new(1000);
    let mut receiver = event_bus.subscribe();

    // Spawn event listener
    tokio::spawn(async move {
        while let Ok(event) = receiver.next().await {
            match event {
                Event::Auth(AuthEvent::AuthenticationStarted { provider }) => {
                    println!("Starting authentication with: {provider}");
                }
                Event::Auth(AuthEvent::AuthenticationInProgress { provider, step }) => {
                    println!("[{provider}] {step}");
                }
                Event::Auth(AuthEvent::AuthenticationSuccess { provider, username, uuid }) => {
                    println!("Authenticated as {username} ({uuid}) via {provider}");
                }
                Event::Auth(AuthEvent::AuthenticationFailed { provider, error }) => {
                    eprintln!("Auth failed for {provider}: {error}");
                }
                _ => {}
            }
        }
    });

    // The device code itself is delivered through the callback, not an
    // event — the callback signature is `Fn(&str, &str) + Send + Sync`.
    let mut auth = MicrosoftAuth::new("your-client-id");
    auth.set_device_code_callback(|code, url| {
        println!("Visit {url} and enter {code}");
    });

    let profile = auth.authenticate(Some(&event_bus)).await?;
    // `Debug` redacts the secret: `access_token: Some("[REDACTED]")`.
    println!("Profile: {profile:?}");

    Ok(())
}
```

## Custom Authenticator

Implement the `Authenticator` trait for your own authentication system.
Note that any provider-issued token must be wrapped in `SecretString`
before being stored on the `UserProfile`, and `AuthEvent` carries
`provider: String` (not the `AuthProvider` enum):

```rust
use lighty_auth::{
    Authenticator, AuthError, AuthProvider, AuthResult, ExposeSecret,
    SecretString, UserProfile, UserRole,
};
use lighty_core::hosts::HTTP_CLIENT;

#[cfg(feature = "events")]
use lighty_event::{AuthEvent, Event, EventBus};

pub struct CustomAuth {
    api_url: String,
    username: String,
    password: String,
}

impl CustomAuth {
    pub fn new(api_url: &str, username: &str, password: &str) -> Self {
        Self {
            api_url: api_url.trim_end_matches('/').to_string(),
            username: username.to_string(),
            password: password.to_string(),
        }
    }
}

impl Authenticator for CustomAuth {
    async fn authenticate(
        &mut self,
        #[cfg(feature = "events")] event_bus: Option<&EventBus>,
    ) -> AuthResult<UserProfile> {
        #[cfg(feature = "events")]
        if let Some(bus) = event_bus {
            bus.emit(Event::Auth(AuthEvent::AuthenticationStarted {
                provider: "Custom".to_string(),
            }));
        }

        let response = HTTP_CLIENT
            .post(format!("{}/api/login", self.api_url))
            .json(&serde_json::json!({
                "username": self.username,
                "password": self.password,
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AuthError::InvalidCredentials);
        }

        let data: serde_json::Value = response.json().await?;

        let uuid = data["uuid"].as_str().unwrap_or("").to_string();
        let username = data["username"]
            .as_str()
            .unwrap_or(&self.username)
            .to_string();

        #[cfg(feature = "events")]
        if let Some(bus) = event_bus {
            bus.emit(Event::Auth(AuthEvent::AuthenticationSuccess {
                provider: "Custom".to_string(),
                username: username.clone(),
                uuid: uuid.clone(),
            }));
        }

        Ok(UserProfile {
            id: data["id"].as_u64(),
            username,
            uuid,
            // The session token MUST be wrapped in `SecretString` so it
            // is redacted from `Debug` and refused by serde.
            access_token: data["token"]
                .as_str()
                .map(|s| SecretString::from(s.to_owned())),
            #[cfg(feature = "keyring")]
            token_handle: None,
            xuid: None,
            email: data["email"].as_str().map(String::from),
            email_verified: data["email_verified"].as_bool().unwrap_or(false),
            money: data["money"].as_f64(),
            role: data["role"].as_object().map(|r| UserRole {
                name: r["name"].as_str().unwrap_or("User").to_string(),
                color: r["color"].as_str().map(String::from),
            }),
            banned: data["banned"].as_bool().unwrap_or(false),
            provider: AuthProvider::Custom { base_url: self.api_url.clone() },
        })
    }
}
```

Custom providers that want OS-keychain routing can mirror the
`with_keyring(service: impl Into<String>)` builder used by `MicrosoftAuth`
/ `AzuriomAuth` and route the token through
`lighty_auth::TokenHandle` (gated behind `#[cfg(feature = "keyring")]`).

## Offline UUID Generation

Generate deterministic UUIDs for offline mode:

```rust
use lighty_auth::generate_offline_uuid;

fn main() {
    // Generate UUID from username
    let uuid1 = generate_offline_uuid("Player123");
    let uuid2 = generate_offline_uuid("Player123");
    let uuid3 = generate_offline_uuid("Different");

    println!("UUID 1: {}", uuid1);
    println!("UUID 2: {}", uuid2); // Same as UUID 1
    println!("UUID 3: {}", uuid3); // Different

    assert_eq!(uuid1, uuid2); // Always the same for same username
    assert_ne!(uuid1, uuid3); // Different for different username
}
```

**How it works**:
- Uses SHA1 hash of username
- Formats as Minecraft-compatible UUID with dashes
- Deterministic: same input always produces same output

## Error Handling

```rust
use lighty_auth::{microsoft::MicrosoftAuth, Authenticator, AuthError};

#[tokio::main]
async fn main() {
    let mut auth = MicrosoftAuth::new("client-id");
    auth.set_device_code_callback(|code, url| {
        println!("Code: {}, URL: {}", code, url);
    });

    match auth.authenticate(None).await {
        Ok(profile) => {
            println!("Success: {}", profile.username);
        }
        Err(AuthError::Network(e)) => {
            eprintln!("Network error: {}", e);
        }
        Err(AuthError::InvalidCredentials) => {
            eprintln!("Invalid username or password");
        }
        Err(AuthError::TwoFactorRequired) => {
            eprintln!("2FA code required");
        }
        Err(AuthError::AccountBanned(who)) => {
            eprintln!("Account is banned: {who}");
        }
        Err(AuthError::DeviceCodeExpired) => {
            eprintln!("Device code expired before user authorised");
        }
        Err(AuthError::Cancelled) => {
            eprintln!("User declined authorisation");
        }
        Err(AuthError::InvalidToken) => {
            eprintln!("Stored refresh token is expired or revoked");
        }
        Err(AuthError::InvalidResponse(msg)) => {
            eprintln!("Provider returned an unexpected payload: {msg}");
        }
        #[cfg(feature = "keyring")]
        Err(AuthError::Keyring(e)) => {
            eprintln!("OS keychain error: {e}");
        }
        Err(e) => {
            eprintln!("Error: {e}");
        }
    }
}
```

## Feature Flags

```toml
[dependencies]
lighty-auth = { version = "26.5.1", features = ["events", "tracing", "keyring"] }
```

Available features:
- `events` - Enables AuthEvent emission (requires lighty-event)
- `tracing` - Enables logging macros
- `keyring` - Enables `MicrosoftAuth::with_keyring(...)` /
  `AzuriomAuth::with_keyring(...)`, the `TokenHandle` type, and the
  `AuthError::Keyring` variant. Pulls D-Bus on Linux, so it stays
  optional for headless / CI builds. From the umbrella crate, enable
  via `lighty-launcher/keyring` (forwarded to both `lighty-auth/keyring`
  and `lighty-launch/keyring`).

## Exports

**In lighty_auth**:
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

**In lighty_launcher**:
```rust
use lighty_launcher::auth::{
    Authenticator,
    UserProfile,
    // ... etc
};
```

## Related Documentation

- [Overview](./overview.md) - Architecture and design
- [Events](./events.md) - AuthEvent types
- [Exports](./exports.md) - Complete export reference
- [Offline](./offline.md) - Offline authentication details
- [Microsoft](./microsoft.md) - Microsoft OAuth flow details
- [Azuriom](./azuriom.md) - Azuriom authentication details
- [Trait](./trait.md) - Implementing custom authenticators

## Related Crates

- **[lighty-event](../../event/README.md)** - Event system
- **[lighty-core](../../core/README.md)** - Hash utilities for offline UUID
- **[lighty-launch](../../launch/README.md)** - Uses UserProfile for launching
