//! Modrinth modpack pinned by `(project, version_id)`.
//!
//! When you don't have a direct CDN URL handy (or want the launcher to
//! always pull the same version even if the URL rotates), use the
//! pinned form: pass a [`ModpackSource::ModrinthPinned`] with the
//! project slug + optional version id. `None` for `version` resolves
//! the latest compatible version via the API.
//!
//! ### How to find a `version_id` for a modpack
//!
//! Same procedure as for individual mods:
//! 1. Open `https://modrinth.com/modpack/<slug>/versions`.
//! 2. Click the target version.
//! 3. The URL becomes `.../version/<version_id>` — copy the trailing segment.
//!
//! - Modrinth modpacks: <https://modrinth.com/modpacks>

use lighty_launcher::mods::ModpackSource;
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

    // Replace `simply-optimized` and `AbCdEfGh` with a real project slug
    // and version_id from modrinth.com before running.
    let source = ModpackSource::ModrinthPinned {
        project: "simply-optimized".into(),
        version: Some("AbCdEfGh".into()),
    };

    let mut instance =
        VersionBuilder::new("modpack-modrinth-pinned", Loader::Fabric, "0.16.9", "1.21.1")
            .with_mod()
                .with_modrinth_modpack(source)
                .done();

    instance
        .launch(&profile, JavaDistribution::Temurin)
        .with_event_bus(&event_bus)
        .run()
        .await?;

    trace_info!("Modrinth pinned modpack launch successful!");

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
