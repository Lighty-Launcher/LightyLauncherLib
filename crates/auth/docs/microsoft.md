# Microsoft OAuth2 Authentication

## Overview

Microsoft authentication implements the **OAuth 2.0 Device Code Flow** to authenticate Minecraft accounts through:
1. Microsoft Identity Platform
2. Xbox Live
3. Minecraft Services

This is the official authentication method for legitimate Minecraft accounts since Microsoft acquired Mojang.

## Prerequisites

### Azure AD Application

You need to register an application in Azure Active Directory:

1. Go to [Azure Portal](https://portal.azure.com)
2. Navigate to **Azure Active Directory** → **App registrations**
3. Click **New registration**
4. Set **Supported account types** to "Accounts in any organizational directory and personal Microsoft accounts"
5. No redirect URI needed (Device Code Flow)
6. After registration, copy the **Application (client) ID**

### Required Permissions

- `XboxLive.signin` - Access to Xbox Live services
- `offline_access` - Refresh tokens (optional but recommended)

## Quick Start

### First launch (device-code prompt)

```rust
use lighty_launcher::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut auth = MicrosoftAuth::new("your-azure-client-id");

    // Set callback to display device code
    auth.set_device_code_callback(|code, url| {
        println!("Please visit: {url}");
        println!("And enter code: {code}");
    });

    // Authenticate (blocks until user completes authorization)
    let profile = auth.authenticate(None).await?;

    println!("Logged in as: {}", profile.username);
    println!("UUID: {}", profile.uuid);
    println!("XUID: {:?}", profile.xuid);
    Ok(())
}
```

### Subsequent launches (silent re-auth)

Once you've persisted the resulting `UserProfile`, use the embedded
refresh token to skip the device-code prompt entirely:

```rust
let mut auth = MicrosoftAuth::new("your-azure-client-id");

// Load the saved profile from your storage (file, keyring, DB…).
let saved: UserProfile = load_from_storage()?;
let AuthProvider::Microsoft { refresh_token: Some(rt), .. } = saved.provider else { /* … */ };

// `rt` is a `SecretString` — pass it as-is, no `expose_secret()` here.
// Zero user interaction — directly hits the Xbox/XSTS/MC chain.
let profile = auth.authenticate_with_refresh_token(&rt, None).await?;
```

See the [Token Management](#token-management) section below for the
full silent-first pattern and storage recommendations.

## Authentication Flow

### Step-by-Step Process

```mermaid
sequenceDiagram
    participant App as Your Application
    participant Auth as MicrosoftAuth
    participant MS as Microsoft OAuth
    participant Xbox as Xbox Live
    participant MC as Minecraft Services

    App->>Auth: authenticate()
    Auth->>MS: POST /devicecode
    MS-->>Auth: device_code, user_code, verification_uri

    Auth->>App: Callback(user_code, verification_uri)
    App->>User: Display "Visit URL and enter CODE"

    loop Every 5 seconds
        Auth->>MS: POST /token (poll)
        MS-->>Auth: "authorization_pending"
    end

    User->>MS: Authorize via browser
    MS-->>Auth: access_token, refresh_token

    Auth->>Xbox: POST /user/authenticate
    Xbox-->>Auth: xbl_token, uhs

    Auth->>Xbox: POST /xsts/authorize
    Xbox-->>Auth: xsts_token

    Auth->>MC: POST /login_with_xbox
    MC-->>Auth: minecraft_token

    Auth->>MC: GET /minecraft/profile
    MC-->>Auth: username, uuid

    Auth-->>App: UserProfile
```

### 1. Request Device Code

```rust
// Endpoint: https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode
// Method: POST

{
    "client_id": "your-client-id",
    "scope": "XboxLive.signin offline_access"
}

// Response:
{
    "user_code": "ABCD1234",
    "device_code": "long-device-code-string",
    "verification_uri": "https://microsoft.com/devicelogin",
    "expires_in": 900,
    "interval": 5
}
```

### 2. Display Code to User

The user must:
1. Visit `https://microsoft.com/devicelogin` in a browser
2. Enter the `user_code` (e.g., "ABCD1234")
3. Sign in with their Microsoft account
4. Authorize the application

### 3. Poll for Access Token

```rust
// Poll every 5 seconds until user authorizes
// Endpoint: https://login.microsoftonline.com/consumers/oauth2/v2.0/token

{
    "grant_type": "urn:ietf:params:oauth:grant-type:device_code",
    "client_id": "your-client-id",
    "device_code": "long-device-code-string"
}

// Responses:
// - "authorization_pending" → Continue polling
// - "authorization_declined" → User declined
// - "expired_token" → Code expired (15 minutes)
// - Success → access_token + refresh_token
```

### 4. Exchange for Xbox Live Token

```rust
// Endpoint: https://user.auth.xboxlive.com/user/authenticate

{
    "Properties": {
        "AuthMethod": "RPS",
        "SiteName": "user.auth.xboxlive.com",
        "RpsTicket": "d=<microsoft_access_token>"
    },
    "RelyingParty": "http://auth.xboxlive.com",
    "TokenType": "JWT"
}

// Response:
{
    "Token": "xbox-live-token",
    "DisplayClaims": {
        "xui": [{
            "uhs": "user-hash-string"
        }]
    }
}
```

### 5. Exchange for XSTS Token

```rust
// Endpoint: https://xsts.auth.xboxlive.com/xsts/authorize

{
    "Properties": {
        "SandboxId": "RETAIL",
        "UserTokens": ["xbox-live-token"]
    },
    "RelyingParty": "rp://api.minecraftservices.com/",
    "TokenType": "JWT"
}

// Response:
{
    "Token": "xsts-token",
    "DisplayClaims": { /* same as above */ }
}
```

**Possible Errors**:
- **2148916233**: Account doesn't own Minecraft
- **2148916238**: Xbox Live not available in user's country

### 6. Exchange for Minecraft Token

```rust
// Endpoint: https://api.minecraftservices.com/authentication/login_with_xbox

{
    "identityToken": "XBL3.0 x=<uhs>;<xsts-token>"
}

// Response:
{
    "access_token": "minecraft-access-token",
    "expires_in": 86400
}
```

### 7. Fetch Minecraft Profile

```rust
// Endpoint: https://api.minecraftservices.com/minecraft/profile
// Headers: Authorization: Bearer <minecraft-access-token>

// Response:
{
    "id": "069a79f444e94726a5befca90e38aaf5",  // UUID without dashes
    "name": "Notch"
}
```

## Configuration

### Polling Interval

Control how often to check for authorization:

```rust
let mut auth = MicrosoftAuth::new(client_id);

// Default: 5 seconds
auth.set_poll_interval(Duration::from_secs(3));  // Faster polling

// Respect server's "interval" field from device code response
```

### Timeout

Set maximum time to wait for user authorization:

```rust
let mut auth = MicrosoftAuth::new(client_id);

// Default: 5 minutes (300 seconds)
auth.set_timeout(Duration::from_secs(600));  // 10 minutes

// Match server's "expires_in" field from device code response
```

### OS Keychain Routing (`with_keyring`)

Opt-in: route the Minecraft access token (and the rotating refresh
token) into the OS keychain instead of keeping them in process memory.
Gated by the `keyring` feature.

```rust
use lighty_launcher::prelude::*;

let mut auth = MicrosoftAuth::new("your-azure-client-id")
    .with_keyring("MyLauncher");

auth.set_device_code_callback(|code, url| {
    println!("Visit {url} and enter: {code}");
});

let profile = auth.authenticate(None).await?;

// `profile.access_token` is now `None` — the token lives in the
// OS keychain. Read it on demand via the handle:
#[cfg(feature = "keyring")]
if let Some(handle) = &profile.token_handle {
    let secret = handle.read()?;          // -> SecretString
    let token  = secret.expose_secret();  // &str, consumed immediately
    /* feed to argv */
}
```

Internally, the token is written under
`service = "MyLauncher"` / `username = "microsoft:{uuid}"` (Keychain on
macOS, Credential Manager on Windows, Secret Service on Linux). The
refresh token stays in `AuthProvider::Microsoft.refresh_token` as a
`SecretString` so the in-process silent-refresh flow doesn't have to
round-trip the keychain on every launch.

### Device Code Callback

Display the code to the user:

```rust
// Console application
auth.set_device_code_callback(|code, url| {
    println!("Please visit: {}", url);
    println!("And enter code: {}", code);
});

// GUI application
auth.set_device_code_callback(|code, url| {
    show_dialog(format!("Visit {} and enter: {}", url, code));
    open_browser(url);  // Optional: auto-open browser
});

// Web application
auth.set_device_code_callback(|code, url| {
    send_to_frontend(json!({
        "type": "device_code",
        "code": code,
        "url": url
    }));
});
```

## Error Handling

```rust
use lighty_auth::{microsoft::MicrosoftAuth, Authenticator, AuthError};

let mut auth = MicrosoftAuth::new(client_id);
auth.set_device_code_callback(|code, url| {
    println!("Visit {} and enter: {}", url, code);
});

match auth.authenticate().await {
    Ok(profile) => {
        println!("Success: {}", profile.username);
    }
    Err(AuthError::DeviceCodeExpired) => {
        eprintln!("Device code expired. Please try again.");
        // Retry authentication
    }
    Err(AuthError::Cancelled) => {
        eprintln!("User declined authorization");
    }
    Err(AuthError::Custom(msg)) if msg.contains("doesn't own Minecraft") => {
        eprintln!("This Microsoft account doesn't own Minecraft");
    }
    Err(AuthError::Custom(msg)) if msg.contains("Xbox Live is not available") => {
        eprintln!("Xbox Live is not available in your country");
    }
    Err(AuthError::Network(e)) => {
        eprintln!("Network error: {}", e);
    }
    Err(e) => {
        eprintln!("Authentication failed: {}", e);
    }
}
```

## Event System Integration

Track authentication progress with events:

```rust
use lighty_auth::{microsoft::MicrosoftAuth, Authenticator};
use lighty_event::{EventBus, Event, AuthEvent};

let event_bus = EventBus::new(1000);
let mut receiver = event_bus.subscribe();

tokio::spawn(async move {
    while let Ok(event) = receiver.next().await {
        if let Event::Auth(auth_event) = event {
            match auth_event {
                AuthEvent::AuthenticationStarted { provider } => {
                    println!("[{}] Starting authentication...", provider);
                }
                AuthEvent::AuthenticationInProgress { provider, step } => {
                    println!("[{}] {}", provider, step);
                }
                AuthEvent::AuthenticationSuccess { username, uuid, .. } => {
                    println!("Success! Logged in as {} ({})", username, uuid);
                }
                AuthEvent::AuthenticationFailed { error, .. } => {
                    eprintln!("Authentication failed: {}", error);
                }
            }
        }
    }
});

let mut auth = MicrosoftAuth::new(client_id);
auth.set_device_code_callback(|code, url| {
    println!("Visit {} and enter: {}", url, code);
});

let profile = auth.authenticate(Some(&event_bus)).await?;
```

**Events Emitted**:

1. `AuthenticationStarted { provider: "Microsoft" }`
2. `AuthenticationInProgress { step: "Requesting device code" }`
3. `AuthenticationInProgress { step: "Waiting for user authorization" }`
4. `AuthenticationInProgress { step: "Exchanging for Xbox Live token" }`
5. `AuthenticationInProgress { step: "Exchanging for XSTS token" }`
6. `AuthenticationInProgress { step: "Exchanging for Minecraft token" }`
7. `AuthenticationInProgress { step: "Fetching Minecraft profile" }`
8. `AuthenticationSuccess { username: "...", uuid: "..." }`

## Token Management

### Silent Re-authentication (Recommended)

The MS refresh token (issued alongside the access token thanks to the
`offline_access` scope) lives **~90 days of inactivity**. Persist it
after the first device-code flow and use it on subsequent launches via
[`MicrosoftAuth::authenticate_with_refresh_token`] to skip the
device-code prompt entirely.

```rust
use lighty_launcher::prelude::*;

let mut auth = MicrosoftAuth::new("your-azure-client-id");

// Try silent first. The refresh_token comes from the previous
// authenticate() call — see "Persistence" below for where to keep it.
let profile = match auth.authenticate_with_refresh_token(&stored_refresh_token, None).await {
    Ok(p) => p,
    Err(_) => {
        // Token expired/revoked → fall back to device-code.
        auth.set_device_code_callback(|code, url| {
            println!("Visit {url} and enter: {code}");
        });
        auth.authenticate(None).await?
    }
};
```

Microsoft **rotates** the refresh token on every refresh (RFC 6749). The
new token always lands in `profile.provider` as
`AuthProvider::Microsoft { refresh_token: Some(_), .. }` — just persist
the whole `UserProfile` again and the next launch picks up the fresh
token automatically.

### Persistence with the OS Keyring

`UserProfile` is intentionally **not** `Serialize` / `Deserialize`, and
its `access_token` is a `SecretString` — dumping a whole profile to
disk is no longer the right pattern. Two recommended setups:

**A. Built-in keyring routing** (recommended). Activate the `keyring`
feature, call `MicrosoftAuth::with_keyring("MyLauncher")` once, and the
provider writes the MC token to the OS keychain automatically (see the
[OS Keychain Routing](#os-keychain-routing-with_keyring) section above).
Your app only has to remember the `uuid` to recover the
[`TokenHandle`](#) on next launch.

**B. Manually persist just the refresh token** for silent re-auth.
Extract it from `profile.provider` and write it where it fits — the OS
keychain is the recommended sink:

```rust
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
```

Backed by Linux Secret Service / macOS Keychain / Windows Credential
Manager — encrypted at rest by the OS, prompts the user on first access.
A complete end-to-end example lives in
[`examples/auth/microsoft.rs`](../../examples/auth/microsoft.rs).

## UserProfile Output

```rust
pub struct UserProfile {
    pub id: None,                                  // No server ID
    pub username: String,                          // Minecraft username
    pub uuid: String,                              // Minecraft UUID (with dashes)
    pub access_token: Some(SecretString),          // MC access token (~24h)
    #[cfg(feature = "keyring")]
    pub token_handle: None,                        // Some(_) only when with_keyring() was called
    pub xuid: Some(String),                        // Xbox User ID, decoded from MC JWT
    pub email: None,                               // Not provided
    pub email_verified: true,                      // Assumed verified
    pub money: None,                               // Not applicable
    pub role: None,                                // Not applicable
    pub banned: false,                             // Checked by Minecraft Services
    pub provider: AuthProvider::Microsoft {        // Drives silent re-auth
        client_id: String,
        refresh_token: Some(SecretString),         // ~90 days, rotates on each use
    },
}
```

With `with_keyring("…")`, `access_token` is `None` and
`token_handle` is `Some(_)`; the secret lives in the OS keychain and is
read on demand via `TokenHandle::read()`.

## Best Practices

### Callback Design

```rust
// ✅ Good: Clear instructions
auth.set_device_code_callback(|code, url| {
    println!("┌─────────────────────────────────────────┐");
    println!("│ Microsoft Account Authentication        │");
    println!("├─────────────────────────────────────────┤");
    println!("│ 1. Visit: {}              │", url);
    println!("│ 2. Enter code: {}                  │", code);
    println!("│ 3. Sign in with your Microsoft account │");
    println!("└─────────────────────────────────────────┘");
});

// ❌ Bad: Unclear or missing callback
auth.set_device_code_callback(|code, _| {
    println!("{}", code);  // User doesn't know what to do
});
```

### Retry Logic

```rust
// ✅ Good: Retry on expiration
let profile = loop {
    match auth.authenticate().await {
        Ok(profile) => break profile,
        Err(AuthError::DeviceCodeExpired) => {
            println!("Code expired. Retrying...");
            continue;
        }
        Err(e) => return Err(e),
    }
};
```

### Testing

```rust
#[tokio::test]
#[ignore]  // Requires user interaction
async fn test_microsoft_auth() {
    let client_id = std::env::var("AZURE_CLIENT_ID").unwrap();
    let mut auth = MicrosoftAuth::new(client_id);

    auth.set_device_code_callback(|code, url| {
        println!("Visit {} and enter: {}", url, code);
    });

    let profile = auth.authenticate().await.unwrap();
    assert!(!profile.username.is_empty());
    assert!(!profile.uuid.is_empty());
    assert!(profile.access_token.is_some());
}
```

## Troubleshooting

### Issue: Device code expires before user completes

**Solution**: Increase timeout

```rust
auth.set_timeout(Duration::from_secs(900));  // 15 minutes
```

### Issue: User doesn't see device code

**Solution**: Ensure callback is set before authenticate()

```rust
auth.set_device_code_callback(|code, url| { /* ... */ });
auth.authenticate().await?;  // Callback will be triggered
```

### Issue: "Account doesn't own Minecraft" error

**Solution**: User needs to purchase Minecraft Java Edition

### Issue: "Xbox Live not available" error

**Solution**: User is in a region where Xbox Live is blocked (e.g., certain countries)

## Performance

Typical authentication duration: **30-60 seconds**

**Breakdown**:
- Device code request: ~500ms
- User authorization: 20-50s (varies by user)
- Token exchange (4 requests): ~2-5s
- Profile fetch: ~200ms

## Security Considerations

- **Client ID is public**: Safe to embed in application
- **No client secret needed**: Device Code Flow doesn't require secrets
- **Tokens are secret-wrapped**: `access_token` / `refresh_token` are
  `SecretString`s — `Debug` prints `[REDACTED]` and the profile cannot
  be `serde`-serialised by accident
- **OS keychain (opt-in)**: `MicrosoftAuth::with_keyring("…")` routes
  the MC token into Keychain / Credential Manager / Secret Service —
  covers core-dump and swap vectors that `SecretString` alone doesn't
- **HTTPS only**: All requests use HTTPS
- **Token expiration**: Minecraft tokens expire after 24 hours; the MS
  refresh token lasts ~90 days of inactivity and rotates per RFC 6749
- See [`AUTH_SECRETS.md`](../../../AUTH_SECRETS.md) for the full threat model
