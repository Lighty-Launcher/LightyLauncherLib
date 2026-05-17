//! Custom auth backend — skeleton showing where to plug your own
//! authentication API while reusing the same OS-keyring "remember me"
//! pattern as the Microsoft / Azuriom examples.
//!
//! This file implements a *minimal* [`Authenticator`] against a
//! hypothetical HTTP API. Replace the URLs and the request/response
//! shapes with your own; the persistence layer below stays identical.
//!
//! Run with:
//! ```bash
//! CUSTOM_AUTH_URL=https://my-launcher-backend.example \
//! CUSTOM_USERNAME=alice \
//! CUSTOM_PASSWORD=hunter2 \
//! cargo run --example auth_custom
//! ```

use lighty_launcher::prelude::*;

const SERVICE: &str = "LightyLauncher";
const ACCOUNT: &str = "default-custom";

// --------- Your auth backend implementation ---------

/// Replace this with whatever shape your backend speaks.
pub struct MyCustomAuth {
    base_url: String,
    username: String,
    password: String,
}

impl MyCustomAuth {
    pub fn new(
        base_url: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            username: username.into(),
            password: password.into(),
        }
    }
}

#[derive(serde::Deserialize)]
struct MyAuthResponse {
    uuid: String,
    username: String,
    access_token: String,
}

impl Authenticator for MyCustomAuth {
    async fn authenticate(
        &mut self,
        _event_bus: Option<&EventBus>,
    ) -> Result<UserProfile, AuthError> {
        // POST {base_url}/login with whatever payload your API expects.
        let url = format!("{}/api/login", self.base_url);
        let client = reqwest::Client::new();
        let resp = client
            .post(&url)
            .json(&serde_json::json!({
                "username": self.username,
                "password": self.password,
            }))
            .send()
            .await
            .map_err(|e| AuthError::Custom(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(AuthError::InvalidCredentials);
        }

        let body: MyAuthResponse = resp
            .json()
            .await
            .map_err(|e| AuthError::InvalidResponse(e.to_string()))?;

        Ok(UserProfile {
            id: None,
            username: body.username,
            uuid: body.uuid,
            access_token: Some(body.access_token),
            xuid: None,
            email: None,
            email_verified: false,
            money: None,
            role: None,
            banned: false,
            provider: AuthProvider::Custom { base_url: self.base_url.clone() },
        })
    }

    async fn verify(&self, token: &str) -> Result<UserProfile, AuthError> {
        let url = format!("{}/api/verify", self.base_url);
        let client = reqwest::Client::new();
        let resp = client
            .post(&url)
            .json(&serde_json::json!({ "access_token": token }))
            .send()
            .await
            .map_err(|e| AuthError::Custom(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(AuthError::InvalidToken);
        }

        let body: MyAuthResponse = resp
            .json()
            .await
            .map_err(|e| AuthError::InvalidResponse(e.to_string()))?;

        Ok(UserProfile {
            id: None,
            username: body.username,
            uuid: body.uuid,
            access_token: Some(body.access_token),
            xuid: None,
            email: None,
            email_verified: false,
            money: None,
            role: None,
            banned: false,
            provider: AuthProvider::Custom { base_url: self.base_url.clone() },
        })
    }
}

// --------- Keyring persistence (identical to the other examples) ---------

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

// --------- Driver ---------

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    AppState::init("LightyLauncher")?;

    let base_url = std::env::var("CUSTOM_AUTH_URL")
        .map_err(|_| anyhow::anyhow!("set CUSTOM_AUTH_URL=https://…"))?;

    // 1) Try silent re-verify from the persisted session token.
    if let Some(saved) = load_profile() {
        if let Some(token) = &saved.access_token {
            let auth = MyCustomAuth::new(&base_url, "", "");
            if let Ok(fresh) = auth.verify(token).await {
                println!("Silent verify OK — welcome back {}", fresh.username);
                save_profile(&fresh)?;
                return Ok(());
            }
            println!("Verify failed, need a fresh login.");
        }
    }

    // 2) Fallback: credentials login.
    let username = std::env::var("CUSTOM_USERNAME")
        .map_err(|_| anyhow::anyhow!("set CUSTOM_USERNAME=…"))?;
    let password = std::env::var("CUSTOM_PASSWORD")
        .map_err(|_| anyhow::anyhow!("set CUSTOM_PASSWORD=…"))?;

    let mut auth = MyCustomAuth::new(&base_url, username, password);
    let profile = auth.authenticate(None).await?;

    save_profile(&profile)?;
    println!("Logged in as {} ({})", profile.username, profile.uuid);
    Ok(())
}
