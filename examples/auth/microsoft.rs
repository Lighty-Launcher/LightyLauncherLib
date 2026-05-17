//! Microsoft auth with persistent "remember me" via the OS keyring.
//!
//! Flow on every launch:
//!
//! 1. Try to load the previous `UserProfile` from the OS keyring.
//! 2. If found, try its stored `refresh_token` for a **silent** Xbox →
//!    XSTS → Minecraft re-auth (no user interaction). The refresh token
//!    lives ~90 days of inactivity.
//! 3. If no profile saved OR the refresh fails (token expired/revoked),
//!    fall back to the **device-code** flow — the user pastes a code
//!    in their browser, just like the first time.
//! 4. Save the resulting `UserProfile` back to the keyring. The new
//!    refresh token (Microsoft rotates it on every refresh) ends up
//!    inside `profile.provider = AuthProvider::Microsoft { .. }`.
//!
//! The keyring crate writes to the platform-native secure store:
//! - Linux  → Secret Service (GNOME Keyring / KWallet)
//! - macOS  → Keychain
//! - Windows → Credential Manager
//!
//! For a real launcher, swap `SERVICE`/`account` for whatever lets you
//! distinguish multiple saved accounts.

use lighty_launcher::prelude::*;

const SERVICE: &str = "LightyLauncher";
const ACCOUNT: &str = "default";

fn load_profile() -> Option<UserProfile> {
    let entry = keyring::Entry::new(SERVICE, ACCOUNT).ok()?;
    let json = entry.get_password().ok()?;
    serde_json::from_str(&json).ok()
}

fn save_profile(profile: &UserProfile) -> anyhow::Result<()> {
    let entry = keyring::Entry::new(SERVICE, ACCOUNT)?;
    entry.set_password(&serde_json::to_string(profile)?)?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    AppState::init("LightyLauncher")?;

    let mut auth = MicrosoftAuth::new("XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX"); // replace with your Azure AD application (client) ID
    auth.set_device_code_callback(|code, url| {
        println!("Visit: {url}");
        println!("Enter code: {code}");
    });

    // 1) Try silent re-auth from the persisted refresh token.
    let profile = match load_profile().and_then(|saved| match saved.provider {
        AuthProvider::Microsoft { refresh_token: Some(rt), .. } => Some((saved.username, rt)),
        _ => None,
    }) {
        Some((username, rt)) => {
            println!("Found saved account `{username}`, trying silent refresh…");
            match auth.authenticate_with_refresh_token(&rt, None).await {
                Ok(p) => {
                    println!("Silent refresh OK — no device code needed.");
                    Some(p)
                }
                Err(e) => {
                    println!("Silent refresh failed ({e}), falling back to device-code.");
                    None
                }
            }
        }
        None => None,
    };

    // 2) Fallback: device-code flow.
    let profile = match profile {
        Some(p) => p,
        None => auth.authenticate(None).await?,
    };

    // 3) Persist the (possibly rotated) refresh token + profile.
    save_profile(&profile)?;
    println!("Logged in as {} ({})", profile.username, profile.uuid);

    Ok(())
}
