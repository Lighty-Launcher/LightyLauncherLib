# Examples

## Basic Examples

### Offline Authentication

```rust
use lighty_auth::{offline::OfflineAuth, Authenticator};

#[tokio::main]
async fn main()  {
    let mut auth = OfflineAuth::new("Steve");
    let profile = auth.authenticate().await?;

    println!("Username: {}", profile.username);
    println!("UUID: {}", profile.uuid);

    Ok(())
}
```

### Microsoft Authentication

```rust
use lighty_auth::{microsoft::MicrosoftAuth, Authenticator};

#[tokio::main]
async fn main()  {
    let mut auth = MicrosoftAuth::new("your-azure-client-id");

    auth.set_device_code_callback(|code, url| {
        println!("Visit {} and enter: {}", url, code);
    });

    let profile = auth.authenticate().await?;
    println!("Logged in as: {}", profile.username);

    Ok(())
}
```

### Azuriom Authentication

```rust
use lighty_auth::{azuriom::AzuriomAuth, Authenticator};

#[tokio::main]
async fn main()  {
    let mut auth = AzuriomAuth::new(
        "https://your-server.com",
        "user@example.com",
        "password123"
    );

    let profile = auth.authenticate().await?;
    println!("Logged in as: {} (Role: {:?})", profile.username, profile.role);

    Ok(())
}
```

## Advanced Examples

### Multi-Provider Launcher

Support multiple authentication methods:

```rust
use lighty_auth::{
    offline::OfflineAuth,
    microsoft::MicrosoftAuth,
    azuriom::AzuriomAuth,
    Authenticator, UserProfile,
};

enum AuthMethod {
    Offline(String),
    Microsoft(String),
    Azuriom { url: String, email: String, password: String },
}

async fn authenticate(method: AuthMethod) -> Result<UserProfile, Box<dyn std::error::Error>> {
    match method {
        AuthMethod::Offline(username) => {
            let mut auth = OfflineAuth::new(username);
            Ok(auth.authenticate().await?)
        }

        AuthMethod::Microsoft(client_id) => {
            let mut auth = MicrosoftAuth::new(client_id);
            auth.set_device_code_callback(|code, url| {
                println!("Visit {} and enter: {}", url, code);
            });
            Ok(auth.authenticate().await?)
        }

        AuthMethod::Azuriom { url, email, password } => {
            let mut auth = AzuriomAuth::new(url, email, password);
            Ok(auth.authenticate().await?)
        }
    }
}

#[tokio::main]
async fn main()  {
    // User selects authentication method
    let method = AuthMethod::Microsoft("client-id".to_string());

    let profile = authenticate(method).await?;
    println!("Authenticated as: {}", profile.username);

    Ok(())
}
```

### Azuriom with 2FA

Handle two-factor authentication:

```rust
use lighty_auth::{azuriom::AzuriomAuth, Authenticator, AuthError};
use std::io::{self, Write};

async fn authenticate_azuriom_with_2fa(
    url: &str,
    email: &str,
    password: &str,
) -> Result<UserProfile, Box<dyn std::error::Error>> {
    let mut auth = AzuriomAuth::new(url, email, password);

    loop {
        match auth.authenticate().await {
            Ok(profile) => return Ok(profile),

            Err(AuthError::TwoFactorRequired) => {
                print!("Enter 2FA code: ");
                io::stdout().flush()?;

                let mut code = String::new();
                io::stdin().read_line(&mut code)?;

                auth.set_two_factor_code(code.trim());
            }

            Err(AuthError::Invalid2FACode) => {
                println!("Invalid 2FA code. Try again.");
                auth.clear_two_factor_code();
            }

            Err(e) => return Err(e.into()),
        }
    }
}

#[tokio::main]
async fn main()  {
    let profile = authenticate_azuriom_with_2fa(
        "https://your-server.com",
        "user@example.com",
        "password123"
    ).await?;

    println!("Successfully logged in as: {}", profile.username);

    Ok(())
}
```

### Token Caching and Verification

Cache an Azuriom session token in the OS keychain (never on disk in
plain text — `UserProfile` is intentionally non-`Serialize`). The
recommended pattern is to store just the token under a service/account
pair, then `verify()` it on next launch:

```rust
use lighty_auth::{azuriom::AzuriomAuth, Authenticator, AuthError, UserProfile, SecretString, ExposeSecret};
use std::path::Path;

const SERVICE: &str = "MyLauncher";
const ACCOUNT: &str = "azuriom-default";

fn save_token(token: &SecretString) -> anyhow::Result<()> {
    let entry = keyring::Entry::new(SERVICE, ACCOUNT)?;
    entry.set_password(token.expose_secret())?;
    Ok(())
}

fn load_token() -> Option<SecretString> {
    let entry = keyring::Entry::new(SERVICE, ACCOUNT).ok()?;
    Some(SecretString::from(entry.get_password().ok()?))
}

async fn authenticate_with_cache(
    url: &str,
    email: &str,
    password: &str,
) -> Result<UserProfile, Box<dyn std::error::Error>> {
    let mut auth = AzuriomAuth::new(url, email, password);

    if let Some(cached) = load_token() {
        match auth.verify(cached.expose_secret()).await {
            Ok(profile) => {
                println!("Using cached authentication");
                return Ok(profile);
            }
            Err(_) => println!("Cached token expired, re-authenticating..."),
        }
    }

    let profile = auth.authenticate(None).await?;
    if let Some(token) = &profile.access_token {
        save_token(token)?;
    }
    Ok(profile)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let profile = authenticate_with_cache(
        "https://your-server.com",
        "user@example.com",
        "password123",
    ).await?;

    println!("Authenticated as: {}", profile.username);
    Ok(())
}
```

For a no-extra-keyring-code variant, use the built-in
`AzuriomAuth::with_keyring("MyLauncher")` (feature `keyring`): the
provider writes the token into the keychain automatically and the
returned `UserProfile.token_handle` reads it on demand.

### Event-Driven Authentication UI

Track authentication progress for UI updates:

```rust
use lighty_auth::{microsoft::MicrosoftAuth, Authenticator};
use lighty_event::{EventBus, Event, AuthEvent};

#[tokio::main]
async fn main()  {
    let event_bus = EventBus::new(1000);
    let mut receiver = event_bus.subscribe();

    // Spawn UI update task
    let ui_task = tokio::spawn(async move {
        while let Ok(event) = receiver.next().await {
            if let Event::Auth(auth_event) = event {
                match auth_event {
                    AuthEvent::AuthenticationStarted { provider } => {
                        println!("🔐 Starting authentication with {}...", provider);
                    }
                    AuthEvent::AuthenticationInProgress { step, .. } => {
                        println!("⏳ {}", step);
                    }
                    AuthEvent::AuthenticationSuccess { username, uuid, .. } => {
                        println!("✅ Success! Logged in as {} ({})", username, uuid);
                    }
                    AuthEvent::AuthenticationFailed { error, .. } => {
                        eprintln!("❌ Authentication failed: {}", error);
                    }
                }
            }
        }
    });

    // Authenticate with event tracking
    let mut auth = MicrosoftAuth::new("your-client-id");
    auth.set_device_code_callback(|code, url| {
        println!("📱 Visit {} and enter: {}", url, code);
    });

    let profile = auth.authenticate(Some(&event_bus)).await?;

    // Wait for UI task to process final events
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    Ok(())
}
```

### Retry on Network Failure

Robust authentication with automatic retry:

```rust
use lighty_auth::{Authenticator, AuthError, UserProfile};
use tokio::time::{sleep, Duration};

async fn authenticate_with_retry<A>(
    auth: &mut A,
    max_retries: u32,
    retry_delay: Duration,
) -> Result<UserProfile, Box<dyn std::error::Error>>
where
    A: Authenticator,
{
    let mut attempts = 0;

    loop {
        match auth.authenticate(None).await {
            Ok(profile) => return Ok(profile),

            Err(AuthError::Network(e)) if attempts < max_retries => {
                attempts += 1;
                eprintln!(
                    "Network error (attempt {}/{}): {}. Retrying in {:?}...",
                    attempts, max_retries, e, retry_delay
                );
                sleep(retry_delay).await;
            }

            Err(e) => return Err(e.into()),
        }
    }
}

#[tokio::main]
async fn main()  {
    use lighty_auth::offline::OfflineAuth;

    let mut auth = OfflineAuth::new("Steve");

    let profile = authenticate_with_retry(
        &mut auth,
        3,  // Max 3 retries
        Duration::from_secs(2),  // Wait 2 seconds between retries
    ).await?;

    println!("Authenticated: {}", profile.username);

    Ok(())
}
```

### Custom Authentication Provider

Implement your own authentication backend:

```rust
use lighty_auth::{
    Authenticator, AuthResult, UserProfile, UserRole, AuthError,
    AuthProvider, SecretString,
};

pub struct CustomAuth {
    api_url: String,
    username: String,
    password: String,
}

impl CustomAuth {
    pub fn new(api_url: String, username: String, password: String) -> Self {
        Self { api_url, username, password }
    }
}

impl Authenticator for CustomAuth {
    async fn authenticate(
        &mut self,
        #[cfg(feature = "events")] event_bus: Option<&lighty_event::EventBus>,
    ) -> AuthResult<UserProfile> {
        // Emit start event
        #[cfg(feature = "events")]
        if let Some(bus) = event_bus {
            bus.emit(lighty_event::Event::Auth(
                lighty_event::AuthEvent::AuthenticationStarted {
                    provider: "Custom".to_string(),
                }
            ));
        }

        // Custom authentication logic
        let response = reqwest::Client::new()
            .post(&format!("{}/auth", self.api_url))
            .json(&serde_json::json!({
                "username": self.username,
                "password": self.password,
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AuthError::InvalidCredentials);
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AuthError::InvalidResponse(e.to_string()))?;

        // Parse response
        let profile = UserProfile {
            id: data["id"].as_u64(),
            username: data["username"]
                .as_str()
                .ok_or_else(|| AuthError::InvalidResponse("Missing username".into()))?
                .to_string(),
            uuid: data["uuid"]
                .as_str()
                .ok_or_else(|| AuthError::InvalidResponse("Missing uuid".into()))?
                .to_string(),
            access_token: data["token"]
                .as_str()
                .map(|s| SecretString::from(s.to_owned())),
            #[cfg(feature = "keyring")]
            token_handle: None,
            xuid: None,
            email: data["email"].as_str().map(|s| s.to_string()),
            email_verified: data["email_verified"].as_bool().unwrap_or(false),
            money: data["balance"].as_f64(),
            role: data["role"].as_str().map(|name| UserRole {
                name: name.to_string(),
                color: Some("#FFFFFF".to_string()),
            }),
            banned: data["banned"].as_bool().unwrap_or(false),
            provider: AuthProvider::Custom { base_url: self.api_url.clone() },
        };

        // Emit success event
        #[cfg(feature = "events")]
        if let Some(bus) = event_bus {
            bus.emit(lighty_event::Event::Auth(
                lighty_event::AuthEvent::AuthenticationSuccess {
                    provider: "Custom".to_string(),
                    username: profile.username.clone(),
                    uuid: profile.uuid.clone(),
                }
            ));
        }

        Ok(profile)
    }
}

#[tokio::main]
async fn main()  {
    let mut auth = CustomAuth::new(
        "https://api.example.com".to_string(),
        "player".to_string(),
        "secret".to_string(),
    );

    let profile = auth.authenticate(None).await?;
    println!("Custom auth successful: {}", profile.username);

    Ok(())
}
```

### Parallel Authentication Attempts

Try multiple auth methods simultaneously:

```rust
use lighty_auth::{
    offline::OfflineAuth,
    microsoft::MicrosoftAuth,
    Authenticator, UserProfile,
};
use tokio::select;

#[tokio::main]
async fn main()  {
    // Create authenticators
    let mut offline = OfflineAuth::new("Player");
    let mut microsoft = MicrosoftAuth::new("client-id");

    microsoft.set_device_code_callback(|code, url| {
        println!("Microsoft: Visit {} and enter {}", url, code);
    });

    // Race both authentication methods
    let profile: UserProfile = select! {
        result = offline.authenticate(None) => {
            println!("Offline authentication completed first");
            result?
        }
        result = microsoft.authenticate(None) => {
            println!("Microsoft authentication completed first");
            result?
        }
    };

    println!("Authenticated as: {}", profile.username);

    Ok(())
}
```

### Session Management

Complete session lifecycle management:

```rust
use lighty_auth::{azuriom::AzuriomAuth, Authenticator, UserProfile, SecretString, ExposeSecret};
use std::time::{Duration, Instant};

struct Session {
    profile: UserProfile,
    token: SecretString,
    expires_at: Instant,
}

impl Session {
    fn new(profile: UserProfile, token: SecretString, duration: Duration) -> Self {
        Self {
            profile,
            token,
            expires_at: Instant::now() + duration,
        }
    }

    fn is_expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }

    fn time_remaining(&self) -> Duration {
        self.expires_at.saturating_duration_since(Instant::now())
    }
}

async fn create_session(
    url: &str,
    email: &str,
    password: &str,
) -> Result<Session, Box<dyn std::error::Error>> {
    let mut auth = AzuriomAuth::new(url, email, password);
    let profile = auth.authenticate(None).await?;

    let token = profile.access_token.clone()
        .ok_or("No access token provided")?;

    Ok(Session::new(
        profile,
        token,
        Duration::from_secs(24 * 60 * 60),  // 24 hours
    ))
}

async fn refresh_session(
    session: &Session,
    auth: &AzuriomAuth,
) -> Result<Session, Box<dyn std::error::Error>> {
    let profile = auth.verify(session.token.expose_secret()).await?;

    Ok(Session::new(
        profile,
        session.token.clone(),
        Duration::from_secs(24 * 60 * 60),
    ))
}

#[tokio::main]
async fn main()  {
    let url = "https://your-server.com";
    let email = "user@example.com";
    let password = "password123";

    // Create session
    let mut session = create_session(url, email, password).await?;
    println!("Session created for: {}", session.profile.username);

    // Use session
    loop {
        if session.is_expired() {
            println!("Session expired. Re-authenticating...");
            session = create_session(url, email, password).await?;
        }

        println!("Time remaining: {:?}", session.time_remaining());

        // Do work with session...

        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
```

## Microsoft Silent Re-auth with OS Keyring

Persist the MS refresh token in the OS keyring so the user never has
to re-enter a device code unless the token expires (≈ 90 days of
inactivity). This is the recommended production pattern for any
Microsoft-based launcher.

`UserProfile` is intentionally non-`Serialize`, so dumping the whole
struct to disk is impossible. Persist **just the refresh token** (a
`SecretString`) instead:

```rust
use lighty_launcher::prelude::*;
use secrecy::{ExposeSecret, SecretString};

const SERVICE: &str = "MyLauncher";
const ACCOUNT: &str = "microsoft-refresh";

fn save_refresh(rt: &SecretString) -> anyhow::Result<()> {
    let entry = keyring::Entry::new(SERVICE, ACCOUNT)?;
    entry.set_password(rt.expose_secret())?;
    Ok(())
}

fn load_refresh() -> Option<SecretString> {
    let entry = keyring::Entry::new(SERVICE, ACCOUNT).ok()?;
    Some(SecretString::from(entry.get_password().ok()?))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut auth = MicrosoftAuth::new("your-azure-client-id");
    auth.set_device_code_callback(|code, url| {
        println!("Visit {url} and enter: {code}");
    });

    // 1) Try silent re-auth from the persisted refresh token.
    let profile = match load_refresh() {
        Some(rt) => match auth.authenticate_with_refresh_token(&rt, None).await {
            Ok(p) => p,
            Err(_) => auth.authenticate(None).await?, // expired → device-code
        },
        None => auth.authenticate(None).await?,       // first launch → device-code
    };

    // 2) Persist the (rotated) refresh token from the fresh profile.
    if let AuthProvider::Microsoft { refresh_token: Some(rt), .. } = &profile.provider {
        save_refresh(rt)?;
    }
    println!("Welcome back, {}", profile.username);
    Ok(())
}
```

For the access token itself, the simplest setup is the built-in keyring
routing — add `.with_keyring("MyLauncher")` to the `MicrosoftAuth` and
the MC access token is written to the OS keychain automatically, with a
[`TokenHandle`](../../auth/docs/exports.md#os-keychain-feature-keyring)
on the returned profile for on-demand reads.

Both paths rely on the [`keyring`](https://crates.io/crates/keyring)
crate (Linux Secret Service / macOS Keychain / Windows Credential
Manager — encrypted at rest by the OS). Enable the `keyring` feature on
`lighty-launcher` (or `lighty-auth` directly) for the built-in routing;
add `keyring = "3"` to your own crate for the manual refresh-token
persistence above.

The full runnable variant lives at
[`examples/auth/microsoft.rs`](../../../examples/auth/microsoft.rs).
Parallel examples for Azuriom (`verify`-based persistence) and a
custom backend (skeleton `Authenticator` impl) are next to it under
`examples/auth/`.

## See Also

- [Overview](./overview.md) - Architecture overview
- [Offline Mode](./offline.md) - Offline authentication
- [Microsoft OAuth](./microsoft.md) - Microsoft authentication, refresh
  tokens, and the silent re-auth flow
- [Azuriom CMS](./azuriom.md) - Azuriom authentication
