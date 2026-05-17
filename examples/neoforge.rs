//! NeoForge launch example. Covers both eras:
//! - MC ≤ 1.20.1 — old `net.neoforged:forge:{mc}-{loader}` artifact path.
//! - MC ≥ 1.20.2 — modern `net.neoforged:neoforge:{loader}` artifact path.
//!
//! `VersionBuilder::new(name, Loader::NeoForge, loader_version, mc_version)`.
//!
//! - Modern (MC ≥ 1.20.2): <https://maven.neoforged.net/releases/net/neoforged/neoforge/>
//! - Old    (MC = 1.20.1): <https://maven.neoforged.net/releases/net/neoforged/forge/>

use lighty_launcher::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(feature = "tracing")]
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    AppState::init("LightyLauncher")?;

    let event_bus = EventBus::new(1000);

    // Authenticate (offline mode — swap for MicrosoftAuth for online play,
    // see examples/auth/microsoft.rs for the full keyring-backed pattern).
    let mut auth = OfflineAuth::new("Player");
    let profile = auth.authenticate(Some(&event_bus)).await?;

    // Build and launch NeoForge instance (1.21.8 + NeoForge 21.8.53)
    let mut neoforge =
        VersionBuilder::new("neoforge-1.21.8", Loader::NeoForge, "21.8.53", "1.21.8");

    neoforge
        .launch(&profile, JavaDistribution::Temurin)
        .with_event_bus(&event_bus)
        .run()
        .await?;

    trace_info!("NeoForge launch successful!");

    let mut receiver = event_bus.subscribe();
    while let Ok(event) = receiver.next().await {
        match event {
            Event::ConsoleOutput(line) => match line.stream {
                ConsoleStream::Stdout => println!("[GAME] {}", line.line),
                ConsoleStream::Stderr => eprintln!("[GAME ERR] {}", line.line),
            },
            Event::InstanceExited(_) => break,
            _ => {}
        }
    }

    trace_info!("Game exited.");

    Ok(())
}
