# Azuriom CMS authentication

[Azuriom](https://azuriom.com) is an open-source CMS for Minecraft
server communities. `AzuriomAuth` talks to its REST API
(`/api/auth/{authenticate,verify,logout}`) so your launcher can sign
players in with the same email + password they use on the website,
including 2FA, role / money / ban metadata, and token verification on
subsequent launches.

```rust
use lighty_auth::{azuriom::AzuriomAuth, Authenticator};

# async fn run() -> anyhow::Result<()> {
let mut auth = AzuriomAuth::new(
    "https://your-server.com",
    "user@example.com",
    "password123",
);

let profile = auth.authenticate(
    #[cfg(feature = "events")] None,
).await?;

println!("{} ({:?})", profile.username, profile.role);
println!("Money: {:?}", profile.money);
# Ok(()) }
```

## Endpoints

| Method | Endpoint | Purpose |
|---|---|---|
| `authenticate(&mut self, …)` | `POST /api/auth/authenticate` | Sign in (with optional 2FA) |
| `verify(&self, token)` | `POST /api/auth/verify` | Check whether a stored token is still valid |
| `logout(&self, token)` | `POST /api/auth/logout` | Invalidate a token server-side |

## Two-factor authentication

The Azuriom API returns the `2fa` error reason when an account has TOTP
enabled. The provider maps it to `AuthError::TwoFactorRequired` — set
the code with `set_two_factor_code` and call `authenticate()` again:

```rust
use lighty_auth::{azuriom::AzuriomAuth, AuthError, Authenticator};

# async fn prompt() -> String { unimplemented!() }
# async fn run() -> anyhow::Result<()> {
let mut auth = AzuriomAuth::new("https://srv.com", "u@e.com", "pw");

loop {
    match auth.authenticate(
        #[cfg(feature = "events")] None,
    ).await {
        Ok(profile) => { println!("hi {}", profile.username); break; }
        Err(AuthError::TwoFactorRequired) => {
            let code = prompt().await;
            auth.set_two_factor_code(code);
        }
        Err(AuthError::Invalid2FACode) => {
            println!("Wrong code, try again");
            auth.clear_two_factor_code();
        }
        Err(e) => return Err(e.into()),
    }
}
# Ok(()) }
```

## Verify / logout

Both methods take the raw token string (call `secret.expose_secret()`
right before passing it in):

```rust
use lighty_auth::{azuriom::AzuriomAuth, AuthError, Authenticator};
use secrecy::ExposeSecret;

# async fn run() -> anyhow::Result<()> {
let mut auth = AzuriomAuth::new("https://srv.com", "u@e.com", "pw");
let profile  = auth.authenticate(
    #[cfg(feature = "events")] None,
).await?;

let secret = profile.access_token.as_ref().expect("token");
let token  = secret.expose_secret();

match auth.verify(token).await {
    Ok(_)                        => println!("still valid"),
    Err(AuthError::InvalidToken) => println!("re-auth needed"),
    Err(e)                       => eprintln!("{e}"),
}

auth.logout(token).await?;
# Ok(()) }
```

## OS keychain routing (`with_keyring`)

Opt-in: route the session token into the OS keychain instead of
keeping it as a `SecretString` in process memory. Gated by the
`keyring` feature.

```rust
use lighty_auth::{azuriom::AzuriomAuth, Authenticator};
use secrecy::ExposeSecret;

# #[cfg(feature = "keyring")]
# async fn run() -> anyhow::Result<()> {
let mut auth = AzuriomAuth::new(
    "https://your-server.com",
    "user@example.com",
    "password123",
)
.with_keyring("MyLauncher");

let profile = auth.authenticate(
    #[cfg(feature = "events")] None,
).await?;

// `profile.access_token` is now `None`; read on demand:
if let Some(handle) = &profile.token_handle {
    let secret = handle.read()?;
    let _token  = secret.expose_secret(); // feed to argv / verify / logout
}
# Ok(()) }
```

Storage key: `service = "MyLauncher"`,
`username = "azuriom:{uuid}"` (Keychain on macOS, Credential Manager
on Windows, Secret Service on Linux).

## Error mapping

Azuriom replies with `{ "status": "error", "reason": "...", "message": "..." }`.
The provider maps reasons as:

| `reason` | `AuthError` |
|---|---|
| `invalid_credentials` | `InvalidCredentials` |
| `2fa` | `TwoFactorRequired` |
| `invalid_2fa` | `Invalid2FACode` |
| `email_not_verified` | `EmailNotVerified` |
| `banned` | `AccountBanned(String)` |
| anything else | `Custom(message)` |

A banned account from the success branch (with `banned: true` in the
payload) is also surfaced as `AccountBanned(username)`.

## Resulting `UserProfile`

```rust
UserProfile {
    id: Some(u64),                       // Azuriom internal ID
    username,                            // display name
    uuid,                                // Minecraft UUID
    access_token: Some(SecretString),    // session token, or None with keyring
    #[cfg(feature = "keyring")]
    token_handle: Option<TokenHandle>,   // Some(_) only with with_keyring()
    xuid: None,
    email: Some(String),                 // mirrors the input email
    email_verified: bool,
    money: Option<f64>,                  // server credits
    role: Option<UserRole { name, color }>,
    banned: false,                       // banned accounts return Err above
    provider: AuthProvider::Azuriom { base_url },
}
```

`Debug` on the profile prints `access_token: Some("[REDACTED]")` —
the `SecretString` wrapper handles it.

## Events

With the `events` feature, Azuriom emits the standard 2-event sequence
(success) or 1+1 (failure). See [events.md](./events.md) for the
catalogue.

## Security notes

- Always use HTTPS — the provider trims trailing `/` but won't add a
  scheme.
- Tokens are `SecretString` (redacted in `Debug`, refused by serde).
- Server-side: passwords are bcrypt-hashed, tokens typically expire
  after 24 h (configurable in Azuriom).
- See `AUTH_SECRETS.md` at the workspace root for the full threat model.

## Related

- [Overview](./overview.md), [How to use](./how-to-use.md)
- [Events](./events.md), [Trait](./trait.md)
- [Microsoft](./microsoft.md), [Offline](./offline.md)
