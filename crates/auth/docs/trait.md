# `Authenticator` trait

The single abstraction every provider implements. Implement it once for
your own backend and the rest of the launcher (launch pipeline, event
bus, examples) treats your provider exactly like the built-in ones.

**Export**: `lighty_auth::Authenticator`.

## Signature

```rust
use std::future::Future;
use lighty_auth::{AuthError, AuthResult, UserProfile};
#[cfg(feature = "events")]
use lighty_event::EventBus;

pub trait Authenticator {
    /// Run the authentication flow and return a fresh profile.
    fn authenticate(
        &mut self,
        #[cfg(feature = "events")] event_bus: Option<&EventBus>,
    ) -> impl Future<Output = AuthResult<UserProfile>> + Send;

    /// Optional: verify an existing access token.
    /// Default impl returns `AuthError::Custom("…")`.
    fn verify(&self, token: &str)
        -> impl Future<Output = AuthResult<UserProfile>> + Send { async move {
            let _ = token;
            Err(AuthError::Custom("Verification not supported".into()))
    }}

    /// Optional: invalidate an access token server-side. Default is no-op.
    fn logout(&self, token: &str)
        -> impl Future<Output = AuthResult<()>> + Send { async move {
            let _ = token;
            Ok(())
    }}
}
```

A few rules worth keeping in mind:

- `&mut self` lets providers cache state (device codes, polling
  cursors, refresh tokens) across retries.
- The `event_bus` parameter only exists when the `events` feature is
  enabled — gate your emissions with `#[cfg(feature = "events")]`.
- The token you put in `UserProfile.access_token` **must** be wrapped
  in `SecretString` — `Debug` and serde rely on this for redaction.
- `AuthEvent` variants carry `provider: String`, not the
  `AuthProvider` enum. The enum is reserved for the profile return
  value (where it captures provider-specific data like the rotating MS
  refresh token).

## Minimal implementation

A thin HTTP-backed example — mirrors
[`examples/auth/custom.rs`](../../../examples/auth/custom.rs):

```rust
use lighty_auth::{
    AuthError, AuthProvider, AuthResult, Authenticator,
    SecretString, UserProfile,
};
use serde::Deserialize;

#[cfg(feature = "events")]
use lighty_event::{AuthEvent, Event, EventBus};

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

#[derive(Deserialize)]
struct LoginResponse {
    uuid: String,
    username: String,
    access_token: String,
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

        let resp = reqwest::Client::new()
            .post(format!("{}/api/login", self.api_url))
            .json(&serde_json::json!({
                "username": self.username,
                "password": self.password,
            }))
            .send()
            .await
            .map_err(|e| AuthError::Custom(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(AuthError::InvalidCredentials);
        }

        let body: LoginResponse = resp
            .json()
            .await
            .map_err(|e| AuthError::InvalidResponse(e.to_string()))?;

        #[cfg(feature = "events")]
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
            // Mandatory: wrap the token in `SecretString`.
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

Custom providers that want OS-keychain routing can mirror the
`with_keyring(service: impl Into<String>)` builder used by
`MicrosoftAuth` / `AzuriomAuth` and route the token through
[`TokenHandle`](./exports.md#os-keychain-feature-keyring) (gated
behind `#[cfg(feature = "keyring")]`).

## Using your provider

Once `Authenticator` is implemented, the rest is mechanical:

```rust
# use lighty_auth::Authenticator;
# struct CustomAuth;
# impl CustomAuth { fn new(_: &str, _: &str, _: &str) -> Self { Self } }
# impl Authenticator for CustomAuth { async fn authenticate(&mut self,
#     #[cfg(feature = "events")] _: Option<&lighty_event::EventBus>)
#     -> lighty_auth::AuthResult<lighty_auth::UserProfile>
# { unimplemented!() } }
# async fn run() -> anyhow::Result<()> {
let mut auth = CustomAuth::new("https://api.example.com", "alice", "hunter2");
let profile = auth.authenticate(
    #[cfg(feature = "events")] None,
).await?;
println!("Authenticated: {}", profile.username);
# Ok(()) }
```

## Related

- [How to use](./how-to-use.md) — patterns for built-in providers
- [Events](./events.md) — `AuthEvent` variants to emit
- [Exports](./exports.md) — full type list
