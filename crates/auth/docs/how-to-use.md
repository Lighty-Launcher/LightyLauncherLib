# How to use `lighty-auth`

The pattern is the same for every provider: instantiate, call
`authenticate(&mut self, …)`, hand the resulting `UserProfile` to the
launch pipeline.

```rust
use lighty_auth::{offline::OfflineAuth, Authenticator};

# async fn run() -> anyhow::Result<()> {
let mut auth = OfflineAuth::new("Player");
let profile  = auth.authenticate(
    #[cfg(feature = "events")] None,
).await?;

println!("{} → {}", profile.username, profile.uuid);
# Ok(()) }
```

Two things to keep in mind for every provider:

- `authenticate` takes `&mut self` — providers cache state (device
  codes, polling cursors, 2FA codes).
- The `event_bus` parameter only exists when the `events` feature is
  enabled. Without `events`: `authenticate().await?`. With: `authenticate(Some(&bus)).await?`
  (or `None`).

For provider-specific snippets, jump to the matching provider doc:
[Offline](./offline.md), [Microsoft](./microsoft.md),
[Azuriom](./azuriom.md), or [Trait](./trait.md) for a custom backend.
Runnable end-to-end programs live under [`examples/auth/`](../../../examples/auth/).

## Picking a provider

| When you want… | Use |
|---|---|
| No network, deterministic UUID, fixtures, tests | [`OfflineAuth`](./offline.md) |
| Premium Minecraft accounts | [`MicrosoftAuth`](./microsoft.md) |
| Email/password on an Azuriom CMS | [`AzuriomAuth`](./azuriom.md) |
| Anything else (LDAP, your own backend, …) | Implement [`Authenticator`](./trait.md) |

## `UserProfile` fields by provider

| Field | Offline | Microsoft | Azuriom |
|---|---|---|---|
| `id` | None | None | Server ID |
| `username` | input | from MC profile | from server |
| `uuid` | derived | from MC profile | from server |
| `access_token` | None | MC token (`SecretString`) | session token (`SecretString`) |
| `xuid` | None | decoded from MC JWT | None |
| `email` | None | None | input email |
| `email_verified` | `false` | `true` | from server |
| `money` | None | None | from server |
| `role` | None | None | from server |
| `banned` | `false` | `false` | from server |
| `provider` | `Offline` | `Microsoft { client_id, refresh_token }` | `Azuriom { base_url }` |

The `provider` variant drives the `${user_type}` launch placeholder:
`Microsoft` → `"msa"`, `Azuriom` → `"mojang"`, `Offline` /
`Custom` → `"legacy"`.

## Tracking progress with events

Enable the `events` feature and pass an `EventBus`. The auth crate
emits `AuthEvent` variants (catalogue in
[events.md](./events.md)) — for the full event-bus mechanics see
[`crates/event/docs/events.md`](../../event/docs/events.md).

```rust
# #[cfg(feature = "events")]
# {
use lighty_auth::{microsoft::MicrosoftAuth, Authenticator};
use lighty_event::{AuthEvent, Event, EventBus};

# async fn run() -> anyhow::Result<()> {
let bus = EventBus::new(1000);
let mut rx = bus.subscribe();

tokio::spawn(async move {
    while let Ok(event) = rx.next().await {
        if let Event::Auth(AuthEvent::AuthenticationInProgress { step, .. }) = event {
            println!("→ {step}");
        }
    }
});

let mut auth = MicrosoftAuth::new("your-client-id");
auth.set_device_code_callback(|code, url| {
    println!("Visit {url} and enter {code}");
});

let _profile = auth.authenticate(Some(&bus)).await?;
# Ok(()) }
# }
```

The device code itself is delivered through the callback signature
`Fn(&str, &str) + Send + Sync`, not through events — the consumer
needs the raw `(code, url)` to display somewhere.

## Persisting tokens (the right way)

`UserProfile` is intentionally **not** `Serialize` / `Deserialize`,
and its `access_token` is a `SecretString`. Dumping it to disk in JSON
is impossible by construction. Two recommended setups:

1. **Built-in OS keychain routing** (recommended, requires `keyring`
   feature). Call `.with_keyring("MyLauncher")` on the provider; the
   token is written into the keychain automatically and the returned
   profile carries a [`TokenHandle`](./exports.md#os-keychain-feature-keyring)
   instead of the raw value. Details:
   [Microsoft](./microsoft.md#os-keychain-routing-with_keyring),
   [Azuriom](./azuriom.md#os-keychain-routing-with_keyring).
2. **Manual refresh-token persistence** (Microsoft only). Extract
   `AuthProvider::Microsoft.refresh_token` and stash it in the OS
   keychain via the `keyring` crate. Pattern shown in
   [microsoft.md → Silent re-auth](./microsoft.md#silent-re-auth).

Full threat model and rationale: `AUTH_SECRETS.md` at the workspace
root.

## Error handling

`AuthError` carries one variant per failure class. The most useful
ones to match individually:

```rust
use lighty_auth::{AuthError, microsoft::MicrosoftAuth, Authenticator};

# async fn run() -> anyhow::Result<()> {
let mut auth = MicrosoftAuth::new("client-id");
auth.set_device_code_callback(|c, u| println!("Visit {u} and enter {c}"));

match auth.authenticate(
    #[cfg(feature = "events")] None,
).await {
    Ok(_)                                => { /* … */ }
    Err(AuthError::InvalidCredentials)   => { /* wrong user/pass */ }
    Err(AuthError::TwoFactorRequired)    => { /* prompt for 2FA */ }
    Err(AuthError::Invalid2FACode)       => { /* retry 2FA */ }
    Err(AuthError::AccountBanned(name))  => { eprintln!("Banned: {name}"); }
    Err(AuthError::DeviceCodeExpired)    => { /* user didn't authorise in time */ }
    Err(AuthError::Cancelled)            => { /* user declined */ }
    Err(AuthError::InvalidToken)         => { /* stored token expired */ }
    Err(AuthError::Network(_))           => { /* offline / DNS / TLS */ }
    Err(AuthError::InvalidResponse(msg)) => { eprintln!("Bad server payload: {msg}"); }
    #[cfg(feature = "keyring")]
    Err(AuthError::Keyring(_))           => { /* OS keychain issue */ }
    Err(e)                               => eprintln!("{e}"),
}
# Ok(()) }
```

## Writing your own provider

Implement [`Authenticator`](./trait.md) — a single async method.
Once that's done, your provider plugs into the launch pipeline and
the event bus exactly like the built-in ones.

## Related

- [Trait](./trait.md) — custom authenticator skeleton
- [Events](./events.md) — `AuthEvent` lifecycle
- [Exports](./exports.md) — full public API
- Provider docs: [Offline](./offline.md), [Microsoft](./microsoft.md), [Azuriom](./azuriom.md)
- [`crates/launch/docs/how-to-use.md`](../../launch/docs/how-to-use.md) — feeding `UserProfile` to a launch
