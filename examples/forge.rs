//! Forge launch example (covers both modern Forge ≥ 1.13 and legacy
//! 1.5.2 → 1.12.2 — the dispatcher picks the right pipeline from the
//! installer's `install_profile.json` schema).
//!
//! `VersionBuilder::new(name, Loader::Forge, loader_version, mc_version)`.
//!
//! - Per-MC builds page:  <https://files.minecraftforge.net/net/minecraftforge/forge/index_{mc}.html>
//! - Recommended/latest:  <https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json>

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

    // Build and launch Forge instance (1.21.8 + Forge 58.1.0 recommended)
    let mut forge = VersionBuilder::new("forge-1.21.8", Loader::Forge, "58.1.0", "1.21.8");

    forge
        .launch(&profile, JavaDistribution::Temurin)
        .with_event_bus(&event_bus)
        .run()
        .await?;

    trace_info!("Forge launch successful!");

    // `run()` returns once the JVM is spawned; the child is owned by a
    // background `tokio::spawn`. Block here until the console handler
    // emits `InstanceExited`, otherwise `main` returns immediately and
    // the tokio runtime tears the child along with it.
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
