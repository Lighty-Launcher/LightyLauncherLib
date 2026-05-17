//! Vanilla Minecraft launch example.
//!
//! `VersionBuilder::new(name, Loader::Vanilla, "", mc_version)`
//! — only `mc_version` is meaningful for vanilla (loader version is `""`).
//!
//! Available MC versions:
//! <https://piston-meta.mojang.com/mc/game/version_manifest_v2.json>

use lighty_launcher::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(feature = "tracing")]
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    AppState::init("LightyLauncher")?;

    // Configure the downloader (optional — defaults are used if this is not called)
    init_downloader_config(DownloaderConfig {
        max_concurrent_downloads: 100, // up from default of 50
        max_retries: 5,                // up from default of 3
        initial_delay_ms: 50,          // up from default of 20ms
    });

    // Authenticate
    let mut auth = OfflineAuth::new("Hamadi");
    #[cfg(feature = "events")]
    let profile = auth.authenticate(None).await?;
    #[cfg(not(feature = "events"))]
    let profile = auth.authenticate().await?;

    let mut version = VersionBuilder::new("vanilla-1.7.2", Loader::Vanilla, "", "1.7.2");

    version
        .launch(&profile, JavaDistribution::Temurin)
        .with_jvm_options()
        .set("Xmx", "4G")
        .set("Xms", "2G")
        .done()
        .with_arguments()
        .set("width", "1920")
        .set("height", "1080")
        .set(KEY_GAME_DIRECTORY, "runtime") //better folder organization
        .done()
        .run()
        .await?;

    trace_info!("Launch successful!");

    Ok(())
}
