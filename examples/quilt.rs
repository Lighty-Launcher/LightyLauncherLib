//! Quilt launch example.
//!
//! `VersionBuilder::new(name, Loader::Quilt, loader_version, mc_version)`.
//! Quilt only ships beta loaders — `0.20.0-beta.9` is the current head.
//!
//! - Supported MC versions: <https://meta.quiltmc.org/v3/versions/game>
//! - Loader for an MC:      <https://meta.quiltmc.org/v3/versions/loader/{mc}>

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

    // Build and launch Quilt instance
    let mut quilt = VersionBuilder::new("quilt-26.1.2", Loader::Quilt, "0.30.0", "26.1.2");

    quilt
        .launch(&profile, JavaDistribution::Temurin)
        .run()
        .await?;

    trace_info!("Quilt launch successful!");

    Ok(())
}
