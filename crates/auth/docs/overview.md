# Overview

`lighty-auth` is the authentication layer of the Lighty Launcher. It
turns whatever credentials your user has (a username, a Microsoft
account, an Azuriom CMS login, or anything you bolt on top) into a
single `UserProfile` that the launch pipeline can feed into
`--username` / `--uuid` / `--accessToken`.

## What this crate gives you

| Item | What it is |
|---|---|
| [`Authenticator`](./trait.md) trait | Single async entry point: `authenticate(&mut self) -> AuthResult<UserProfile>` |
| `UserProfile` | Unified return type, with the session token wrapped in `SecretString` |
| [`OfflineAuth`](./offline.md) | Deterministic UUID, no network |
| [`MicrosoftAuth`](./microsoft.md) | OAuth 2.0 Device Code Flow + silent refresh |
| [`AzuriomAuth`](./azuriom.md) | Email / password (+ 2FA) against an Azuriom CMS |
| `TokenHandle` (feature `keyring`) | Opt-in OS-keychain handle, returned in place of the raw token |

The full public surface (types, traits, re-exports) is enumerated in
[`exports.md`](./exports.md); the events lifecycle in
[`events.md`](./events.md); the usage pattern and snippets in
[`how-to-use.md`](./how-to-use.md).

## How a `UserProfile` is built

```text
   ┌──────────────┐    authenticate()    ┌─────────────┐
   │ Credentials  │ ───────────────────► │ UserProfile │
   └──────────────┘                      └─────────────┘
   username / OAuth /                      username + uuid (required)
   email+password                          access_token: SecretString
                                           xuid, role, money, …
                                           provider: AuthProvider
```

`UserProfile` is **not** `Serialize` / `Deserialize`. The session token
sits inside a [`SecretString`](https://docs.rs/secrecy/) so:

- `Debug` prints `[REDACTED]` for the token.
- `serde_json::to_string(&profile)` is a compile error.
- Reading the token at launch time goes through
  `secret.expose_secret()` — explicit, audit-friendly.

For "remember me" without storing secrets on disk, the `keyring`
feature routes tokens into the OS keychain via
`MicrosoftAuth::with_keyring(...)` / `AzuriomAuth::with_keyring(...)`
and returns a `TokenHandle` on the profile instead of the raw token.
Full threat model and design rationale: see `AUTH_SECRETS.md` at the
workspace root.

## Provider summary

| Provider | Network | Latency | Use case |
|---|---|---|---|
| `OfflineAuth` | None | < 1 ms | Tests, doctests, LAN, offline play |
| `MicrosoftAuth` | 6-8 round-trips | 30-60 s (user wait) | Premium Minecraft accounts |
| `AzuriomAuth` | 1-2 round-trips | 100-500 ms | Custom CMS-backed servers |
| Custom | yours | yours | Implement [`Authenticator`](./trait.md) |

Field availability per provider is summarised in
[how-to-use.md](./how-to-use.md#userprofile-fields-by-provider).

## Cargo features

| Feature | Adds |
|---|---|
| `events` | `AuthEvent` emission through [`lighty-event`](../../event/docs/events.md) |
| `tracing` | `tracing` logs at the provider level |
| `keyring` | `TokenHandle`, `with_keyring(...)` on Microsoft / Azuriom, `AuthError::Keyring` |

`keyring` pulls a D-Bus dependency on Linux, so it stays optional for
headless / CI builds. From the umbrella crate, enable via
`lighty-launcher/keyring` (forwarded to both `lighty-auth/keyring` and
`lighty-launch/keyring`).

## Related docs

- [How to use](./how-to-use.md) — common patterns and snippets
- [Trait](./trait.md) — implementing a custom authenticator
- [Events](./events.md) — `AuthEvent` lifecycle
- [Exports](./exports.md) — full public API surface
- Provider-specific: [Offline](./offline.md), [Microsoft](./microsoft.md), [Azuriom](./azuriom.md)
- Runnable programs: [`examples/auth/`](../../../examples/auth/)
