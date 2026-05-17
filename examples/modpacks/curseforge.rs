//! CurseForge modpack pinned by `(project_id, file_id)`.
//!
//! Both IDs are required (CurseForge has no "latest pack version"
//! shortcut for modpacks — you always pin a specific file). The launch
//! crate downloads the `.zip` archive, parses `manifest.json`,
//! resolves each `(projectID, fileID)` listed in the pack to its
//! download URL via the CurseForge API, then downloads the mods and
//! merges the `overrides/` directory into the instance's runtime.
//!
//! Requires `CURSEFORGE_API_KEY` to be set via
//! [`lighty_launcher::mods::curseforge::set_api_key`] BEFORE `.run()`.
//! Get a key at <https://console.curseforge.com/?#/api-keys>.
//!
//! ### How to find a modpack `project_id` / `file_id`
//!
//! - `project_id` is shown as "Project ID" in the About sidebar at
//!   `https://www.curseforge.com/minecraft/modpacks/<slug>`.
//! - `file_id` is the trailing URL segment after clicking a file under
//!   the Files tab: `.../modpacks/<slug>/files/<file_id>`.

use lighty_launcher::mods::curseforge;
use lighty_launcher::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(feature = "tracing")]
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    AppState::init("LightyLauncher")?;

    let event_bus = EventBus::new(1000);

    // CurseForge API key — read from env to keep secrets out of source.
    curseforge::set_api_key(std::env::var("CURSEFORGE_API_KEY")?);

    let mut auth = OfflineAuth::new("Player");
    let profile = auth.authenticate(Some(&event_bus)).await?;

    // Replace `644795` / `5234567` with real ids from curseforge.com
    // (Minecraft → Modpacks → click your pack → Files → click a file).
    let mut instance =
        VersionBuilder::new("modpack-curseforge", Loader::Forge, "58.1.0", "1.21.8")
            .with_mod()
                .with_curseforge_modpack(644795, 5234567)
                .done();

    instance
        .launch(&profile, JavaDistribution::Temurin)
        .with_event_bus(&event_bus)
        .run()
        .await?;

    trace_info!("CurseForge modpack launch successful!");

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
