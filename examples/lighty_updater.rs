//! LightyUpdater launch example.
//!
//! `LightyVersionBuilder::new(name, server_url)` — the MC version
//! and loader are resolved from the remote server's metadata at install
//! time, not specified locally.
//!
//! The server must expose the LightyUpdater protocol (a JSON manifest
//! describing the loader + libraries + assets to fetch).

use lighty_launcher::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(feature = "tracing")]
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    AppState::init("LightyLauncher")?;

    // Authenticate
    let mut auth = OfflineAuth::new("Hamadi");
    #[cfg(feature = "events")]
    let profile = auth.authenticate(None).await?;
    #[cfg(not(feature = "events"))]
    let profile = auth.authenticate().await?;

    // Configure LightyUpdater server URL
    let url = "http://dev.polargames";

    // Build LightyUpdater instance
    let mut version = LightyVersionBuilder::new("wynlers-dev", url);

    // Fetch metadata to verify connection
    let _metadata = version.get_metadata().await?;
    trace_info!("LightyUpdater metadata fetched successfully");

    // Launch the game
    version.launch(&profile, JavaDistribution::Temurin)
        .run()
        .await?;

    trace_info!("LightyUpdater launch successful!");


    Ok(())
}
