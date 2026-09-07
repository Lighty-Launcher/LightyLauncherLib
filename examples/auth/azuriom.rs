//! Azuriom auth with persistent "remember me" via the OS keyring.
//!
//! Flow on every launch:
//!
//! 1. Try to load the previous session token from the OS keyring.
//! 2. If found, call `verify(access_token)` — the Azuriom API will
//!    either confirm the session (return a fresh profile) or reject
//!    it (token revoked / changed password / expired server-side).
//! 3. If no profile saved OR verify fails, prompt for credentials and
//!    call `authenticate()`. In a real launcher you'd show a login UI;
//!    here we read email/password from env vars to keep the example
//!    headless.
//! 4. Persist the resulting session token so the next launch is silent.
//!
//! Run with:
//! ```bash
//! AZURIOM_URL=https://example.com \
//! AZURIOM_EMAIL=user@example.com \
//! AZURIOM_PASSWORD=hunter2 \
//! cargo run --example auth_azuriom
//! ```

use lighty_launcher::auth::{ExposeSecret, SecretString};
use lighty_launcher::prelude::*;

const SERVICE: &str = "LightyLauncher";
const ACCOUNT: &str = "default-azuriom";

fn load_token() -> Option<SecretString> {
    let entry = keyring::Entry::new(SERVICE, ACCOUNT).ok()?;
    Some(SecretString::from(entry.get_password().ok()?))
}

fn save_token(token: &SecretString) -> anyhow::Result<()> {
    let entry = keyring::Entry::new(SERVICE, ACCOUNT)?;
    entry.set_password(token.expose_secret())?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    AppState::init("LightyLauncher")?;

    let base_url = std::env::var("AZURIOM_URL")
        .map_err(|_| anyhow::anyhow!("set AZURIOM_URL=https://your-site.example"))?;

    // 1) Try silent re-verify from the persisted session token.
    if let Some(saved) = load_token() {
        // `verify` doesn't need credentials, so build a stub authenticator.
        let auth = AzuriomAuth::new(&base_url, "", "");
        match auth.verify(saved.expose_secret()).await {
            Ok(fresh) => {
                println!("Silent verify OK — welcome back {}", fresh.username);
                if let Some(token) = &fresh.access_token {
                    save_token(token)?;
                }
                return Ok(());
            }
            Err(e) => println!("Verify failed ({e}), need a fresh login."),
        }
    }

    // 2) Fallback: credentials login.
    let email = std::env::var("AZURIOM_EMAIL")
        .map_err(|_| anyhow::anyhow!("set AZURIOM_EMAIL=…"))?;
    let password = std::env::var("AZURIOM_PASSWORD")
        .map_err(|_| anyhow::anyhow!("set AZURIOM_PASSWORD=…"))?;

    let mut auth = AzuriomAuth::new(&base_url, email, password);

    // Handle 2FA: if the first call returns TwoFactorRequired, ask for
    // the code and retry. A real UI would loop here.
    let profile = match auth.authenticate(None).await {
        Ok(p) => p,
        Err(AuthError::TwoFactorRequired) => {
            let code = std::env::var("AZURIOM_2FA")
                .map_err(|_| anyhow::anyhow!("2FA required — set AZURIOM_2FA=……"))?;
            auth.set_two_factor_code(code);
            auth.authenticate(None).await?
        }
        Err(e) => return Err(e.into()),
    };

    if let Some(token) = &profile.access_token {
        save_token(token)?;
    }
    println!("Logged in as {} ({})", profile.username, profile.uuid);
    Ok(())
}
