# Offline Authentication

## Overview

Offline authentication provides a network-free authentication mode suitable for:
- **Testing and Development** - No external dependencies
- **Offline Play** - Single-player or LAN games
- **Cracked Launchers** - Non-premium Minecraft clients (use responsibly)

## Features

- ✅ **No Network Required** - Works without internet connection
- ✅ **Deterministic UUIDs** - Same username always generates the same UUID
- ✅ **Fast** - Authentication completes in microseconds
- ✅ **Username Validation** - Enforces Minecraft username rules

## Quick Start

```rust
use lighty_auth::{offline::OfflineAuth, Authenticator};

#[tokio::main]
async fn main()  {
    let mut auth = OfflineAuth::new("Steve");
    let profile = auth.authenticate().await?;

    println!("Username: {}", profile.username);
    println!("UUID: {}", profile.uuid);
    println!("Access Token: {:?}", profile.access_token); // None

    Ok(())
}
```

## Username Validation Rules

The offline authenticator enforces Minecraft username requirements:

### Length Requirements

- **Minimum**: 3 characters
- **Maximum**: 16 characters

```rust
// ✅ Valid
let mut auth = OfflineAuth::new("Steve");
let profile = auth.authenticate().await?;

// ❌ Invalid: Too short
let mut auth = OfflineAuth::new("Ab");
let result = auth.authenticate().await;
assert!(result.is_err());

// ❌ Invalid: Too long
let mut auth = OfflineAuth::new("ThisUsernameIsTooLong");
let result = auth.authenticate().await;
assert!(result.is_err());
```

### Character Requirements

- **Allowed**: Letters (a-z, A-Z), numbers (0-9), underscore (_)
- **Not Allowed**: Spaces, special characters, emojis

```rust
// ✅ Valid usernames
OfflineAuth::new("Steve");
OfflineAuth::new("Player123");
OfflineAuth::new("Cool_Gamer");
OfflineAuth::new("ABC_123_xyz");

// ❌ Invalid usernames
OfflineAuth::new("Player 123");    // Contains space
OfflineAuth::new("Player-123");    // Contains hyphen
OfflineAuth::new("Player@123");    // Contains @
OfflineAuth::new("Émilie");        // Contains accented character
```

## UUID Generation

### Algorithm

Offline UUIDs are generated using **UUID v5** (SHA1-based):

```rust
pub fn generate_offline_uuid(username: &str) -> String {
    // Namespace: "OfflinePlayer:"
    const NAMESPACE: &[u8] = b"OfflinePlayer:";

    // Concatenate namespace and username
    let data = [NAMESPACE, username.as_bytes()].concat();

    // Calculate SHA1 hash
    let hash = sha1(&data);

    // Format as UUID v5
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-5{:01x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        hash[0], hash[1], hash[2], hash[3],     // Time low
        hash[4], hash[5],                       // Time mid
        hash[6] & 0x0f, hash[7],                // Time high + version
        (hash[8] & 0x3f) | 0x80, hash[9],       // Clock seq + variant
        hash[10], hash[11], hash[12], hash[13], hash[14], hash[15]  // Node
    )
}
```

### Properties

- **Deterministic**: Same username → same UUID
- **Standard Compliant**: Follows RFC 4122 UUID v5 specification
- **Unique**: Different usernames produce different UUIDs
- **Compatible**: Works with Minecraft protocol

### Examples

```rust
use lighty_auth::generate_offline_uuid;

let uuid1 = generate_offline_uuid("Steve");
let uuid2 = generate_offline_uuid("Steve");
let uuid3 = generate_offline_uuid("Alex");

assert_eq!(uuid1, uuid2);  // Same username → same UUID
assert_ne!(uuid1, uuid3);  // Different username → different UUID

println!("Steve: {}", uuid1);
// Output: "f3c8d69b-0000-5000-8000-00000000000a"

println!("Alex: {}", uuid3);
// Output: "a1b2c3d4-0000-5000-8000-00000000000b"
```

## UserProfile Output

The `UserProfile` returned by offline authentication:

```rust
pub struct UserProfile {
    pub id: None,                          // No server ID
    pub username: String,                  // Input username
    pub uuid: String,                      // Generated UUID
    pub access_token: None,                // No session token
    #[cfg(feature = "keyring")]
    pub token_handle: None,                // Never set in offline mode
    pub xuid: None,
    pub email: None,                       // No email
    pub email_verified: false,             // Not verified
    pub money: None,                       // No credits
    pub role: None,                        // No role
    pub banned: false,                     // Not banned
    pub provider: AuthProvider::Offline,
}
```

### Direct constructor

When you only need a placeholder profile (tests, doctests, fixtures,
fallback paths) skipping the `OfflineAuth` validation step is fine.
The crate exposes a minimal public constructor:

```rust
use lighty_auth::UserProfile;

let profile = UserProfile::offline("Steve", "069a79f4-44e9-4726-a5be-fca90e38aaf5");
// All optional fields default to `None`; `provider` is `AuthProvider::Offline`.
```

Note: this skips the `OfflineAuth` username-validation rules above. Use
the `OfflineAuth` flow whenever the username comes from real user input.

## Event System Integration

With the `events` feature enabled:

```rust
use lighty_auth::{offline::OfflineAuth, Authenticator};
use lighty_event::{EventBus, Event, AuthEvent};

let event_bus = EventBus::new(1000);
let mut receiver = event_bus.subscribe();

tokio::spawn(async move {
    while let Ok(event) = receiver.next().await {
        if let Event::Auth(auth_event) = event {
            match auth_event {
                AuthEvent::AuthenticationStarted { provider } => {
                    println!("Starting authentication with {}", provider);
                }
                AuthEvent::AuthenticationSuccess { username, uuid, .. } => {
                    println!("Success: {} ({})", username, uuid);
                }
                AuthEvent::AuthenticationFailed { error, .. } => {
                    eprintln!("Failed: {}", error);
                }
                _ => {}
            }
        }
    }
});

let mut auth = OfflineAuth::new("Steve");
let profile = auth.authenticate(Some(&event_bus)).await?;
```

**Events Emitted**:

1. `AuthenticationStarted { provider: "Offline" }`
2. `AuthenticationSuccess { provider: "Offline", username: "Steve", uuid: "..." }`

Or on error:

1. `AuthenticationStarted { provider: "Offline" }`
2. `AuthenticationFailed { provider: "Offline", error: "Username must be between 3 and 16 characters" }`

## Error Handling

```rust
use lighty_auth::{offline::OfflineAuth, Authenticator, AuthError};

let mut auth = OfflineAuth::new("AB");  // Too short

match auth.authenticate().await {
    Ok(profile) => {
        println!("Authenticated: {}", profile.username);
    }
    Err(AuthError::InvalidCredentials) => {
        eprintln!("Username cannot be empty");
    }
    Err(AuthError::Custom(msg)) => {
        eprintln!("Validation error: {}", msg);
    }
    Err(e) => {
        eprintln!("Unexpected error: {}", e);
    }
}
```

## Use Cases

### 1. Development and Testing

```rust
#[tokio::test]
async fn test_game_launch() {
    let mut auth = OfflineAuth::new("TestPlayer");
    let profile = auth.authenticate().await.unwrap();

    // Use profile for testing game launch
    assert_eq!(profile.username, "TestPlayer");
    assert!(!profile.uuid.is_empty());
}
```

### 2. LAN Multiplayer

```rust
// Each player on LAN can use offline mode
let mut auth1 = OfflineAuth::new("Player1");
let mut auth2 = OfflineAuth::new("Player2");

let profile1 = auth1.authenticate().await?;
let profile2 = auth2.authenticate().await?;

// Each player gets a unique UUID
assert_ne!(profile1.uuid, profile2.uuid);
```

### 3. Fallback Authentication

```rust
// Try Microsoft auth, fallback to offline
let profile = match microsoft_auth.authenticate(None).await {
    Ok(profile) => profile,
    Err(_) => {
        eprintln!("Microsoft auth failed, using offline mode");
        let mut offline = OfflineAuth::new("Player");
        offline.authenticate(None).await?
    }
};
```

## Limitations

### No Token Management

- **No `verify()` support**: Cannot verify sessions
- **No `logout()` support**: No sessions to invalidate
- **No access token**: `access_token` is always `None`

```rust
let mut auth = OfflineAuth::new("Steve");

// ✅ Works
let profile = auth.authenticate().await?;

// ❌ Not supported
let verify_result = auth.verify("some-token").await;
assert!(verify_result.is_err());
```

### Not Suitable for Online Servers

- No server-side validation
- No skin/cape support
- No account ownership verification
- May be blocked by server anti-cheat

## Performance

Offline authentication is extremely fast:

```rust
use std::time::Instant;

let start = Instant::now();

let mut auth = OfflineAuth::new("Steve");
let profile = auth.authenticate().await?;

let duration = start.elapsed();
println!("Authentication took: {:?}", duration);
// Typical output: "Authentication took: 50µs" (microseconds)
```

**Breakdown**:
- Username validation: ~1µs
- SHA1 hashing: ~20µs
- UUID formatting: ~10µs
- Profile creation: ~20µs
- **Total**: ~50µs

## Best Practices

### Username Input Validation

```rust
// ✅ Good: Validate before creating authenticator
fn create_offline_auth(username: &str) -> Result<OfflineAuth, String> {
    if username.len() < 3 || username.len() > 16 {
        return Err("Username must be 3-16 characters".into());
    }

    if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err("Username can only contain letters, numbers, and underscore".into());
    }

    Ok(OfflineAuth::new(username))
}

// ❌ Bad: Let authenticator validate (less user-friendly)
let mut auth = OfflineAuth::new(user_input);
auth.authenticate().await?;  // Generic error message
```

### UUID Caching

```rust
// ✅ Good: Cache generated UUIDs
use std::collections::HashMap;

struct OfflineUUIDCache {
    cache: HashMap<String, String>,
}

impl OfflineUUIDCache {
    fn get_or_generate(&mut self, username: &str) -> String {
        self.cache
            .entry(username.to_string())
            .or_insert_with(|| generate_offline_uuid(username))
            .clone()
    }
}
```

### Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_valid_usernames() {
        let usernames = vec!["Steve", "Player123", "Cool_Gamer"];

        for username in usernames {
            let mut auth = OfflineAuth::new(username);
            let result = auth.authenticate().await;
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_invalid_usernames() {
        let usernames = vec!["AB", "TooLongUsername123", "Player 123"];

        for username in usernames {
            let mut auth = OfflineAuth::new(username);
            let result = auth.authenticate().await;
            assert!(result.is_err());
        }
    }
}
```
