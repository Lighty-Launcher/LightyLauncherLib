# Authenticator Trait

## Overview

The `Authenticator` trait is the core interface for all authentication providers in `lighty-auth`. Implement this trait to create custom authentication systems.

**Export**: `lighty_auth::Authenticator`

## Trait Definition

```rust
pub trait Authenticator {
    /// Run the authentication flow and return a fresh profile.
    /// The `event_bus` parameter only exists when the `events`
    /// feature is enabled.
    fn authenticate(
        &mut self,
        #[cfg(feature = "events")] event_bus: Option<&EventBus>,
    ) -> impl Future<Output = AuthResult<UserProfile>> + Send;

    /// Check whether an existing access token is still valid.
    /// Default impl returns `AuthError::Custom("Verification not supported")`.
    fn verify(&self, token: &str)
        -> impl Future<Output = AuthResult<UserProfile>> + Send { /* default */ }

    /// Invalidate an access token server-side. Default impl is a no-op.
    fn logout(&self, token: &str)
        -> impl Future<Output = AuthResult<()>> + Send { /* default */ }
}
```

`AuthEvent` variants carry the provider name as a plain `String`
(`"Microsoft"`, `"Azuriom"`, `"Offline"`, or your own label) — **not**
the `AuthProvider` enum. The enum is reserved for the `UserProfile`
return value, where it captures provider-specific data (e.g. the MS
refresh token).

`UserProfile.access_token` is a [`SecretString`](https://docs.rs/secrecy/)
(re-exported as `lighty_auth::SecretString`). Custom authenticators
must wrap their token in `SecretString::from(token)` before constructing
the profile — see the example below. `Debug` redacts it automatically
and the profile cannot be serialised in plain JSON.

## Implementing Custom Authenticator

A minimal implementation against a hypothetical HTTP backend — closely
mirroring [`examples/auth/custom.rs`](../../../examples/auth/custom.rs):

```rust
use lighty_launcher::prelude::*;

pub struct CustomAuth {
    api_url: String,
    username: String,
    password: String,
}

impl CustomAuth {
    pub fn new(
        api_url: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        Self {
            api_url: api_url.into().trim_end_matches('/').to_string(),
            username: username.into(),
            password: password.into(),
        }
    }
}

#[derive(serde::Deserialize)]
struct CustomAuthResponse {
    uuid: String,
    username: String,
    access_token: String,
}

impl Authenticator for CustomAuth {
    async fn authenticate(
        &mut self,
        event_bus: Option<&EventBus>,
    ) -> AuthResult<UserProfile> {
        if let Some(bus) = event_bus {
            bus.emit(Event::Auth(AuthEvent::AuthenticationStarted {
                provider: "Custom".to_string(),
            }));
        }

        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{}/api/login", self.api_url))
            .json(&serde_json::json!({
                "username": self.username,
                "password": self.password,
            }))
            .send()
            .await
            .map_err(|e| AuthError::Custom(e.to_string()))?;

        if !resp.status().is_success() {
            if let Some(bus) = event_bus {
                bus.emit(Event::Auth(AuthEvent::AuthenticationFailed {
                    provider: "Custom".to_string(),
                    error: "Invalid credentials".to_string(),
                }));
            }
            return Err(AuthError::InvalidCredentials);
        }

        let body: CustomAuthResponse = resp
            .json()
            .await
            .map_err(|e| AuthError::InvalidResponse(e.to_string()))?;

        if let Some(bus) = event_bus {
            bus.emit(Event::Auth(AuthEvent::AuthenticationSuccess {
                provider: "Custom".to_string(),
                username: body.username.clone(),
                uuid: body.uuid.clone(),
            }));
        }

        Ok(UserProfile {
            id: None,
            username: body.username,
            uuid: body.uuid,
            access_token: Some(SecretString::from(body.access_token)),
            #[cfg(feature = "keyring")]
            token_handle: None,
            xuid: None,
            email: None,
            email_verified: false,
            money: None,
            role: None,
            banned: false,
            provider: AuthProvider::Custom { base_url: self.api_url.clone() },
        })
    }
}

```

## Usage

```rust
use lighty_launcher::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut auth = CustomAuth::new(
        "https://api.example.com",
        "alice",
        "hunter2",
    );

    // With events: pass Some(&event_bus); without, pass None.
    let profile = auth.authenticate(None).await?;
    println!("Authenticated: {}", profile.username);
    Ok(())
}
```

## Related Documentation

- [How to Use](./how-to-use.md) - Examples of using built-in authenticators
- [Events](./events.md) - AuthEvent types to emit
- [Exports](./exports.md) - All exported types
