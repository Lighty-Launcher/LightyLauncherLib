# Events

`lighty-auth` emits the `AuthEvent` family through the bus exposed by
`lighty-event`. Enable the `events` feature on `lighty-auth` to opt
in.

The full event catalogue across all modules (Launch, Loader, Java,
Auth, Console, …) lives in
[`crates/event/docs/events.md`](../../event/docs/events.md). This page
only documents what **auth** specifically emits.

**Export**: `lighty_event::AuthEvent`, also re-exported as
`lighty_launcher::event::AuthEvent`.

A note on the `provider` field: every variant carries
`provider: String` (`"Microsoft"`, `"Azuriom"`, `"Offline"`, or
whatever label your custom provider passes) — **not** the
`AuthProvider` enum. The enum is reserved for the `UserProfile` return
value, where it carries provider-specific data (e.g. the MS refresh
token).

## Variants

```rust
# use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum AuthEvent {
    AuthenticationStarted    { provider: String },
    AuthenticationInProgress { provider: String, step: String },
    AuthenticationSuccess    { provider: String, username: String, uuid: String },
    AuthenticationFailed     { provider: String, error: String },
    AlreadyAuthenticated     { provider: String, username: String },
}
```

| Variant | When |
|---|---|
| `AuthenticationStarted` | At the very start of every `authenticate()` |
| `AuthenticationInProgress` | Between stages of a multi-step flow (see sequences below) |
| `AuthenticationSuccess` | Right before `authenticate()` returns `Ok` |
| `AuthenticationFailed` | When the flow errors out (`error` = human-readable) |
| `AlreadyAuthenticated` | Emitted by higher-level callers (silent-refresh helpers, session reuse) — not by built-in providers |

`AuthenticationSuccess` carries the user's `username` and `uuid`
— **never** the access token. Secret material stays in the returned
`UserProfile` (`SecretString`) or in the OS keychain when
`with_keyring(...)` is active.

The Microsoft device code itself is delivered through the
`MicrosoftAuth::set_device_code_callback(|code, url| { … })` callback,
not through events — the consumer needs the raw values.

## Sequences

### Offline

```text
AuthenticationStarted   { provider: "Offline" }
AuthenticationSuccess   { provider: "Offline", username, uuid }
```

### Microsoft — device code (success)

```text
AuthenticationStarted    { provider: "Microsoft" }
AuthenticationInProgress { step: "Requesting device code" }
AuthenticationInProgress { step: "Waiting for user authorization" }
AuthenticationInProgress { step: "Exchanging for Xbox Live token" }
AuthenticationInProgress { step: "Exchanging for XSTS token" }
AuthenticationInProgress { step: "Exchanging for Minecraft token" }
AuthenticationInProgress { step: "Fetching Minecraft profile" }
AuthenticationSuccess    { username, uuid }
```

### Microsoft — silent refresh

```text
AuthenticationStarted    { provider: "Microsoft" }
AuthenticationInProgress { step: "Refreshing Microsoft token" }
AuthenticationInProgress { step: "Exchanging for Xbox Live token" }
…
AuthenticationSuccess    { username, uuid }
```

### Azuriom

```text
AuthenticationStarted   { provider: "Azuriom" }
AuthenticationSuccess   { provider: "Azuriom", username, uuid }
# or on error:
AuthenticationFailed    { provider: "Azuriom", error }
```

## Listening

```rust
# #[cfg(feature = "events")]
# {
use lighty_auth::{offline::OfflineAuth, Authenticator};
use lighty_event::{AuthEvent, Event, EventBus};

# async fn run() -> anyhow::Result<()> {
let bus = EventBus::new(1000);
let mut rx = bus.subscribe();

tokio::spawn(async move {
    while let Ok(event) = rx.next().await {
        if let Event::Auth(auth_event) = event {
            match auth_event {
                AuthEvent::AuthenticationStarted    { provider }              => println!("[{provider}] start"),
                AuthEvent::AuthenticationInProgress { provider, step }        => println!("[{provider}] {step}"),
                AuthEvent::AuthenticationSuccess    { provider, username, .. }=> println!("[{provider}] OK {username}"),
                AuthEvent::AuthenticationFailed     { provider, error }       => eprintln!("[{provider}] {error}"),
                AuthEvent::AlreadyAuthenticated     { provider, username }    => println!("[{provider}] reused {username}"),
            }
        }
    }
});

let mut auth = OfflineAuth::new("Player");
let _profile = auth.authenticate(Some(&bus)).await?;
# Ok(()) }
# }
```

## Related

- [How to use](./how-to-use.md) — `with_event_bus` patterns
- [Exports](./exports.md) — feature gate details
- [Event catalogue](../../event/docs/events.md) — all modules
- [Launch events](../../launch/docs/events.md) — `LaunchEvent` /
  `ModloaderEvent` emitted by the launch pipeline
