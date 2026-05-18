# Offline authentication

`OfflineAuth` is the zero-network provider. It validates the username,
derives a deterministic UUID v5 from `"OfflinePlayer:" + username`, and
returns a `UserProfile` with no session token. Useful for tests,
LAN play and as a fallback when an online provider fails.

```rust,no_run
use lighty_auth::{offline::OfflineAuth, Authenticator};

# async fn run() -> anyhow::Result<()> {
let mut auth = OfflineAuth::new("Steve");
let profile = auth.authenticate(
    #[cfg(feature = "events")] None,
).await?;

println!("{} → {}", profile.username, profile.uuid);
// access_token is always `None` in offline mode.
# Ok(()) }
```

## Username rules

`authenticate()` rejects anything that wouldn't survive the Minecraft
client side:

| Rule | Detail |
|---|---|
| Length | 3 – 16 characters |
| Allowed | `a–z`, `A–Z`, `0–9`, `_` |
| Rejected | spaces, hyphens, accented chars, emojis, `@`, … |

Violations return `AuthError::Custom(msg)` (length / charset) or
`AuthError::InvalidCredentials` (empty input).

## Deterministic UUIDs

Same username, same UUID — every time, on every machine. The helper
[`generate_offline_uuid`](./exports.md) is also re-exported if you
need the UUID without going through `authenticate()`:

```rust,no_run
use lighty_auth::generate_offline_uuid;

let a = generate_offline_uuid("Steve");
let b = generate_offline_uuid("Steve");
let c = generate_offline_uuid("Alex");

assert_eq!(a, b);
assert_ne!(a, c);
```

## Bypass for fixtures

When you only need a placeholder profile (doctests, unit fixtures,
fallback paths), skip the validator and build a profile directly:

```rust,no_run
use lighty_auth::UserProfile;

let profile = UserProfile::offline("Steve", "069a79f4-44e9-4726-a5be-fca90e38aaf5");
// All optional fields = None, provider = AuthProvider::Offline.
```

This skips the username validation above — use the `OfflineAuth` flow
whenever the username comes from real user input.

## Events

With the `events` feature, `OfflineAuth` emits the standard pair (see
[events.md](./events.md) for the full catalogue):

```text
AuthenticationStarted   { provider: "Offline" }
AuthenticationSuccess   { provider: "Offline", username, uuid }
```

## Related

- [Overview](./overview.md)
- [How to use](./how-to-use.md)
- [Events](./events.md)
