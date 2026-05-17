# Events

## Overview

`lighty-auth` emits `AuthEvent` types through the event bus system provided by `lighty-event`. These events track authentication flow progress.

**Feature**: Requires `events` feature flag

**Export**:
- Event types: `lighty_event::AuthEvent`
- Re-export: `lighty_launcher::event::AuthEvent`

The `provider` field on every variant is a plain `String`
(`"Microsoft"`, `"Azuriom"`, `"Offline"`, or whatever label a custom
authenticator emits) — **not** the `AuthProvider` enum. The enum is
reserved for the `UserProfile` return value where it carries
provider-specific data (e.g. the secret-wrapped MS refresh token).

## AuthEvent Types

### AuthenticationStarted

Emitted at the very start of an `authenticate()` call.

**Fields**:
- `provider: String` — provider label

```rust
use lighty_event::{EventBus, Event, AuthEvent};
use lighty_auth::{offline::OfflineAuth, Authenticator};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let event_bus = EventBus::new(1000);
    let mut receiver = event_bus.subscribe();

    tokio::spawn(async move {
        while let Ok(event) = receiver.next().await {
            if let Event::Auth(AuthEvent::AuthenticationStarted { provider }) = event {
                println!("Starting authentication with: {}", provider);
            }
        }
    });

    let mut auth = OfflineAuth::new("Player");
    auth.authenticate(Some(&event_bus)).await?;

    Ok(())
}
```

### AuthenticationInProgress

Emitted between stages of a multi-step flow (Microsoft device-code /
Xbox / XSTS / Minecraft / Profile, Azuriom request, etc.).

**Fields**:
- `provider: String`
- `step: String` — short human-readable label of the current stage

```rust
if let Event::Auth(AuthEvent::AuthenticationInProgress { provider, step }) = event {
    println!("[{provider}] {step}");
}
```

For Microsoft, the device code itself is delivered through the
`MicrosoftAuth::set_device_code_callback(|code, url| { … })` callback —
not through an event — because the consumer needs the raw values
(callback signature `Fn(&str, &str) + Send + Sync`).

### AuthenticationSuccess

Emitted right after the provider has produced a valid `UserProfile`,
just before `authenticate()` returns.

**Fields**:
- `provider: String`
- `username: String`
- `uuid: String`

```rust
if let Event::Auth(AuthEvent::AuthenticationSuccess { provider, username, uuid }) = event {
    println!("Logged in as {username} ({uuid}) via {provider}");
}
```

Note the event carries the user's `username` / `uuid` but **never** the
access token — secret material stays in the returned `UserProfile`
(secret-wrapped via [`SecretString`](https://docs.rs/secrecy/)) or in
the OS keychain when `with_keyring(...)` is active.

### AuthenticationFailed

Emitted when the flow errors out.

**Fields**:
- `provider: String`
- `error: String`

```rust
if let Event::Auth(AuthEvent::AuthenticationFailed { provider, error }) = event {
    eprintln!("Auth failed for {provider}: {error}");
}
```

### AlreadyAuthenticated

Emitted by higher-level callers (e.g. silent-refresh helpers) that
short-circuit when a still-valid session is reused.

**Fields**:
- `provider: String`
- `username: String`

## Complete Event Flow

### Offline

```
AuthenticationStarted
    ↓
AuthenticationSuccess
```

### Microsoft (device-code, success)

```
AuthenticationStarted              { provider: "Microsoft" }
AuthenticationInProgress           { step: "Requesting device code" }
AuthenticationInProgress           { step: "Waiting for user authorization" }
AuthenticationInProgress           { step: "Exchanging for Xbox Live token" }
AuthenticationInProgress           { step: "Exchanging for XSTS token" }
AuthenticationInProgress           { step: "Exchanging for Minecraft token" }
AuthenticationInProgress           { step: "Fetching Minecraft profile" }
AuthenticationSuccess              { username, uuid }
```

### Microsoft (silent refresh)

```
AuthenticationStarted              { provider: "Microsoft" }
AuthenticationInProgress           { step: "Refreshing Microsoft token" }
AuthenticationInProgress           { step: "Exchanging for Xbox Live token" }
…
AuthenticationSuccess              { username, uuid }
```

### Microsoft (failure)

```
AuthenticationStarted
    ↓
AuthenticationInProgress (any step)
    ↓
AuthenticationFailed
```

### Azuriom

```
AuthenticationStarted
    ↓
AuthenticationSuccess  or  AuthenticationFailed
```

## Related Documentation

- [How to Use](./how-to-use.md) - Practical authentication examples with events
- [Exports](./exports.md) - Complete export reference
- [lighty-event Events](../../event/docs/events.md) - All event types
