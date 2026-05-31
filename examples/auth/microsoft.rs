//! Microsoft auth with persistent "remember me" via the OS keyring.
//!
//! **Azure AD setup**: before running this example you need a properly
//! configured Azure app registration (public client, mobile platform enabled)
//! and your client ID must be approved by Mojang to call Minecraft Services.
//! Full step-by-step guide: <https://hamadi.gitbook.io/lightylauncher/auth/application-id-microsoft>
//!
//! Flow on every launch:
//!
//! 1. Try to load the previous refresh token from the OS keyring.
//! 2. If found, try a **silent** Xbox → XSTS → Minecraft re-auth (no user
//!    interaction). The refresh token lives ~90 days of inactivity.
//! 3. If no token saved OR the refresh fails (expired/revoked), fall back
//!    to the **device-code** flow — the user pastes a code in their browser.
//! 4. Save the (possibly rotated) refresh token back to the keyring.
//!
//! The keyring crate writes to the platform-native secure store:
//! - Linux  → Secret Service (GNOME Keyring / KWallet)
//! - macOS  → Keychain
//! - Windows → Credential Manager

use lighty_launcher::prelude::*;
use lighty_launcher::auth::{ExposeSecret, SecretString};

const SERVICE: &str = "LightyLauncher";
const ACCOUNT: &str = "microsoft_refresh_token";

fn load_refresh_token() -> Option<SecretString> {
    let entry = keyring::Entry::new(SERVICE, ACCOUNT).ok()?;
    let token = entry.get_password().ok()?;
    Some(SecretString::from(token))
}

fn save_refresh_token(token: &SecretString) -> anyhow::Result<()> {
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

    // Replace with your Azure AD application (client) ID.
    // Your app must be approved by Mojang — see setup guide in the header above.#azure-ad-setup
    let mut auth = MicrosoftAuth::new("XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX");
    auth.set_device_code_callback(|code, url| {
        println!("Visit: {url}");
        println!("Enter code: {code}");
    });

    // 1) Try silent re-auth from the persisted refresh token.
    let profile = match load_refresh_token() {
        Some(rt) => {
            println!("Found saved refresh token, trying silent re-auth…");
            match auth.authenticate_with_refresh_token(&rt, None).await {
                Ok(profile) => {
                    println!("Silent refresh OK — no device code needed.");
                    Some(profile)
                }
                Err(err) => {
                    println!("Silent refresh failed ({err}), falling back to device-code.");
                    None
                }
            }
        }
        None => None,
    };

    // 2) Fallback: device-code flow.
    let profile = match profile {
        Some(profile) => profile,
        None => auth.authenticate(None).await?,
    };

    // 3) Persist the (possibly rotated) refresh token.
    if let AuthProvider::Microsoft { refresh_token: Some(rt), .. } = &profile.provider {
        save_refresh_token(rt)?;
    }

    println!("Logged in as {} ({})", profile.username, profile.uuid);

    Ok(())
}
