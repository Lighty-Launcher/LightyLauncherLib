# Microsoft OAuth 2.0 authentication

`MicrosoftAuth` runs the **Device Code Flow** against Microsoft
Identity → Xbox Live → XSTS → Minecraft Services and returns a real
Minecraft session token for premium accounts. Once a user has signed
in, the embedded refresh token enables a silent re-auth on subsequent
launches so the device code prompt only appears once per ~90 days of
inactivity.

```rust
use lighty_auth::{microsoft::MicrosoftAuth, Authenticator};

# async fn run() -> anyhow::Result<()> {
let mut auth = MicrosoftAuth::new("your-azure-client-id");

auth.set_device_code_callback(|code, url| {
    println!("Visit {url} and enter: {code}");
});

let profile = auth.authenticate(
    #[cfg(feature = "events")] None,
).await?;

println!("{} ({})", profile.username, profile.uuid);
// profile.provider carries the rotating refresh token for silent
// re-auth on subsequent launches — see "Silent re-auth" below.
# Ok(()) }
```

## Azure AD setup

The launcher does the OAuth dance, but you still need an Azure AD app
registration — and your client ID must be approved by Mojang before
it can call Minecraft Services.

**→ Full step-by-step guide with screenshots:
[application-id-microsoft.md](./application-id-microsoft.md)**

### Step 1 — Create the app registration

[Azure Portal](https://portal.azure.com) → **App registrations** →
**New registration**.

![Step 1 — New registration](../../../../assets/microsoft_auth/step_1.png)

Fill in a display name for your launcher, then under **Supported account
types** select **"Personal Microsoft accounts only"** — Minecraft accounts
are consumer accounts, not work/school accounts.

![Step 2 — Personal Microsoft accounts only](../../../../assets/microsoft_auth/step_2.png)

Leave the redirect URI empty and click **Register**.

### Step 2 — Copy your client ID

On the **Overview** page after creation, copy the **Application (client) ID**
— this is the value you pass to `MicrosoftAuth::new(...)`.

![Step 3 — Copy client ID](../../../../assets/microsoft_auth/step_3.png)

Then click **Authentication** in the left menu.

### Step 3 — Add the desktop platform

On the **Authentication** page, click **Add a platform** →
**Mobile and desktop applications**.

![Step 4 — Select platform](../../../../assets/microsoft_auth/step_4.png)

Tick `https://login.microsoftonline.com/common/oauth2/nativeclient` and
click **Configure**.

![Step 5 — nativeclient URI](../../../../assets/microsoft_auth/step_5.png)

### Step 4 — Enable public client flows

Still on **Authentication**, go to the **Parameters** tab and set
**Allow public client flows** to **Yes**, then click **Save**.

![Step 6 — Allow public client flows](../../../../assets/microsoft_auth/step_6.png)

> This toggle is the most commonly missed step. Without it you get
> `AADSTS70002: The provided client is not supported for this feature`.

### Step 5 — Submit for Mojang approval

**Your client ID will be rejected by Minecraft Services until Mojang
approves it.** Submit your app at:
<https://aka.ms/mce-reviewappid>

You will need your **Application (client) ID** and your
**Directory (tenant) ID** (both visible on the Overview page).
Submissions are reviewed weekly.

![Step 7 — AppID review form](../../../../assets/microsoft_auth/step_7.png)

> Without this approval you will see:
> `"errorMessage": "Invalid app registration, see https://aka.ms/AppRegInfo"`

Required scopes are hardcoded by the provider: `XboxLive.signin
offline_access`. The `offline_access` scope is what unlocks the
refresh token.

> Full protocol reference: [Minecraft Wiki — Microsoft authentication](https://minecraft.wiki/w/Microsoft_authentication)

## The token chain

```text
Microsoft device code ─► MS access_token (+ refresh_token)
                         │
                         ▼
                       Xbox Live token
                         │
                         ▼
                       XSTS token (+ user hash)
                         │
                         ▼
                       Minecraft token (~24h)
                         │
                         ▼
                       Minecraft profile (name + uuid)
```

The provider emits an `AuthEvent::AuthenticationInProgress` at every
arrow with the `events` feature enabled — see
[events.md](./events.md#sequences).

Known failures and their meaning:

| Error | Meaning | Fix |
|---|---|---|
| `AADSTS70002` | Public client flows not enabled | Authentication page → Parameters → Allow public client flows → Yes |
| `Invalid app registration` | Client ID not approved by Mojang | Submit at [aka.ms/mce-reviewappid](https://aka.ms/mce-reviewappid) |
| XSTS `2148916233` | Account doesn't own Minecraft Java Edition | — |
| XSTS `2148916238` | Xbox Live unavailable in the user's country | — |

## Silent re-auth

After the first device-code flow, the returned `UserProfile` carries
the refresh token inside `AuthProvider::Microsoft.refresh_token`. On
the next launch, hand it to
[`MicrosoftAuth::authenticate_with_refresh_token`] to skip the prompt
entirely.

```rust
use lighty_auth::{
    microsoft::MicrosoftAuth, AuthProvider, Authenticator, SecretString,
};

# async fn run(stored_refresh_token: SecretString) -> anyhow::Result<()> {
let mut auth = MicrosoftAuth::new("your-azure-client-id");

// 1) Try silent re-auth first.
let profile = match auth
    .authenticate_with_refresh_token(
        &stored_refresh_token,
        #[cfg(feature = "events")] None,
    )
    .await
{
    Ok(p) => p,
    Err(_) => {
        // 2) Token expired (~90 days) or revoked → fall back to device-code.
        auth.set_device_code_callback(|code, url| {
            println!("Visit {url} and enter: {code}");
        });
        auth.authenticate(
            #[cfg(feature = "events")] None,
        ).await?
    }
};

// 3) Persist the new refresh token — Microsoft rotates it on every
//    refresh per RFC 6749.
if let AuthProvider::Microsoft { refresh_token: Some(_rt), .. } = &profile.provider {
    // save_refresh(_rt)?;
}
# Ok(()) }
```

A complete keyring-backed example lives at
[`examples/auth/microsoft.rs`](../../../examples/auth/microsoft.rs).

## OS keychain routing (`with_keyring`)

Opt-in: route the **Minecraft access token** into the OS keychain
instead of keeping it as a `SecretString` in process memory. Gated by
the `keyring` feature.

```rust
use lighty_auth::{microsoft::MicrosoftAuth, Authenticator};
use secrecy::ExposeSecret;

# #[cfg(feature = "keyring")]
# async fn run() -> anyhow::Result<()> {
let mut auth = MicrosoftAuth::new("your-azure-client-id")
    .with_keyring("MyLauncher");

auth.set_device_code_callback(|code, url| {
    println!("Visit {url} and enter: {code}");
});

let profile = auth.authenticate(
    #[cfg(feature = "events")] None,
).await?;

// `profile.access_token` is now `None` — the token lives in the
// OS keychain. Read it on demand via the handle:
if let Some(handle) = &profile.token_handle {
    let secret = handle.read()?;          // -> SecretString
    let _token  = secret.expose_secret(); // &str, feed to argv
}
# Ok(()) }
```

Storage key: `service = "MyLauncher"`, `username = "microsoft:{uuid}"`
(Keychain on macOS, Credential Manager on Windows, Secret Service on
Linux). The refresh token stays in `AuthProvider::Microsoft` as a
`SecretString` so the silent-refresh flow doesn't round-trip the
keychain on every launch.

## Configuration knobs

| Method | Default | Purpose |
|---|---|---|
| `set_device_code_callback(Fn(code, url))` | none — warning logged | Show the device code to the user |
| `set_poll_interval(Duration)` | 5 s | How often to poll the token endpoint |
| `set_timeout(Duration)` | 5 min | Give up if the user never authorises |
| `with_keyring(service)` | off | Route the MC token through the OS keychain |

## Error handling

```rust
use lighty_auth::{AuthError, microsoft::MicrosoftAuth, Authenticator};

# async fn run() -> anyhow::Result<()> {
let mut auth = MicrosoftAuth::new("client-id");
auth.set_device_code_callback(|code, url| {
    println!("Visit {url} and enter: {code}");
});

match auth.authenticate(
    #[cfg(feature = "events")] None,
).await {
    Ok(_profile) => { /* … */ }
    Err(AuthError::DeviceCodeExpired) => { /* retry with a fresh code */ }
    Err(AuthError::Cancelled)         => { /* user declined */ }
    Err(AuthError::Custom(msg)) if msg.contains("doesn't own Minecraft") => { /* … */ }
    Err(AuthError::Custom(msg)) if msg.contains("Xbox Live")            => { /* … */ }
    Err(AuthError::InvalidToken) => { /* stored refresh token expired */ }
    Err(AuthError::Network(_))   => { /* offline / DNS */ }
    Err(e) => eprintln!("{e}"),
}
# Ok(()) }
```

## Resulting `UserProfile`

```rust
UserProfile {
    id: None,
    username,                              // from /minecraft/profile
    uuid,                                  // dashed UUID
    access_token: Some(SecretString),      // MC token, ~24 h, or None with keyring
    #[cfg(feature = "keyring")]
    token_handle: Option<TokenHandle>,     // Some(_) only with with_keyring()
    xuid: Some(String),                    // decoded from MC JWT (may be None on parse failure)
    email: None,
    email_verified: true,
    money: None,
    role: None,
    banned: false,                         // pre-filtered by Minecraft Services
    provider: AuthProvider::Microsoft {
        client_id,
        refresh_token: Some(SecretString), // ~90 days, rotates per RFC 6749
    },
}
```

## Security notes

- Client ID is **public** — safe to embed in your binary.
- No client secret — Device Code Flow doesn't need one.
- All tokens are `SecretString` (redacted in `Debug`, refused by serde).
- HTTPS only — every endpoint is fixed and TLS-only.
- See `AUTH_SECRETS.md` at the workspace root for the full threat model.

## Related

- [Overview](./overview.md), [How to use](./how-to-use.md)
- [Events](./events.md) — including the silent-refresh sequence
- [Azuriom](./azuriom.md), [Offline](./offline.md)
