//! Modrinth modpack via direct CDN URL.
//!
//! The simplest modpack flow: paste a `.mrpack` URL into
//! `.with_modrinth_modpack(...)`. The launch crate downloads the
//! archive into the launcher's cache, parses `modrinth.index.json`,
//! validates each download URL against the Modrinth domain whitelist,
//! and merges the listed mods + the `overrides/` tree into the
//! instance's runtime directory.
//!
//! ### How to find a `.mrpack` URL
//!
//! 1. Browse <https://modrinth.com/modpacks>.
//! 2. On a pack's page, open "Versions" → click a version.
//! 3. Right-click the download button → "Copy link address".
//!    The URL points at `https://cdn.modrinth.com/.../<file>.mrpack`.
//!
//! The MC version + loader declared in the manifest are authoritative —
//! the values passed to `VersionBuilder::new(...)` here are just
//! placeholders that will be overridden (with a `trace_warn!`) if the
//! pack targets a different combo.
//!
//! - Modrinth modpacks: <https://modrinth.com/modpacks>
//! - .mrpack format:    <https://docs.modrinth.com/modpacks/format>

use lighty_launcher::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(feature = "tracing")]
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    AppState::init("LightyLauncher")?;

    let event_bus = EventBus::new(1000);

    let mut auth = OfflineAuth::new("Player");
    let profile = auth.authenticate(Some(&event_bus)).await?;

    let mut instance =
        VersionBuilder::new("modpack-modrinth-url", Loader::Fabric, "0.16.9", "1.21.1")
            .with_mod()
                // Replace with a real .mrpack CDN URL from modrinth.com:
                .with_modrinth_modpack(
                    "https://cdn.modrinth.com/data/<project>/versions/<version>/<pack>.mrpack",
                )
                .done();

    instance
        .launch(&profile, JavaDistribution::Temurin)
        .with_event_bus(&event_bus)
        .with_arguments()
            .set(KEY_GAME_DIRECTORY, "runtime")
            .done()
        .run()
        .await?;

    trace_info!("Modrinth modpack launch successful!");

    // Drain events until the JVM exits.
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

    Ok(())
}
