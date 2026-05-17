//! Fabric launch example.
//!
//! `VersionBuilder::new(name, Loader::Fabric, loader_version, mc_version)`.
//!
//! - Supported MC versions: <https://meta.fabricmc.net/v2/versions/game>
//! - Loader versions:       <https://meta.fabricmc.net/v2/versions/loader>
//! - Compatibility per MC:  <https://meta.fabricmc.net/v2/versions/loader/{mc}>

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

    let mut fabric = VersionBuilder::new("fabric-26.1.2", Loader::Fabric, "0.19.2", "26.1.2");

    fabric
        .launch(&profile, JavaDistribution::Zulu)
        .with_arguments()
        .set(KEY_GAME_DIRECTORY, "runtime") //better folder organization
        .done()
        .run()
        .await?;

    trace_info!("Fabric launch successful!");

    Ok(())
}
