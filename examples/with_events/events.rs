//! End-to-end events + instance-management example (vanilla MC).
//!
//! Demonstrates the full event bus (auth → install → JRE → loader →
//! extraction → console → exit). The per-domain event handlers live
//! in [`handlers`] — one `.rs` per event family (auth, launch, java,
//! loader, core, instance, console).
//!
//! `VersionBuilder::new(name, Loader::Vanilla, "", mc_version)`.
//!
//! Available MC versions:
//! <https://piston-meta.mojang.com/mc/game/version_manifest_v2.json>
//!
//! ```bash
//! cargo run --example with_events --features vanilla,events,tracing
//! ```

mod handlers;

use lighty_launcher::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(feature = "tracing")]
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    AppState::init("LightyLauncher")?;

    // Event bus — every install / launch / console line will flow
    // through `handlers::spawn_logger`, dispatched to the right
    // per-domain `.rs` file under `handlers/`.
    let event_bus = EventBus::new(1000);
    handlers::spawn_logger(&event_bus);

    // Authenticate (offline)
    let mut auth = OfflineAuth::new("Player");
    let profile = auth.authenticate(Some(&event_bus)).await?;

    let mut instance = VersionBuilder::new("events", Loader::Vanilla, "", "1.21.1");

    instance
        .launch(&profile, JavaDistribution::Temurin)
        .with_event_bus(&event_bus)
        .with_jvm_options()
        .set("Xmx", "2G")
        .set("Xms", "1G")
        .done()
        .with_arguments()
        .set("width", "1280")
        .set("height", "720")
        .set(KEY_GAME_DIRECTORY, "runtime") //better folder organization
        .done()
        .run()
        .await?;

    // Keep the runtime alive until the JVM exits — otherwise
    // `#[tokio::main]` would return immediately after the spawn and
    // the event-logger task would never get the `[GAME] ...` lines.
    let mut receiver = event_bus.subscribe();
    while let Ok(event) = receiver.next().await {
        if matches!(event, Event::InstanceExited(_)) {
            break;
        }
    }

    trace_info!("Launch successful!");
    Ok(())
}
