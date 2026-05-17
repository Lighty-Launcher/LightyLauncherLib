//! CurseForge mod fetching example (Fabric on MC 1.21.1).
//!
//! Chain `.with_mod().with_curseforge_mods(...)` on the `VersionBuilder`
//! to pull a list of mods from CurseForge. The tuple is `(mod_id, file_id?)`:
//!
//! - `None` resolves the latest release compatible with the instance's
//!   MC + loader. Required dependencies are walked transitively.
//! - `Some(file_id)` pins a specific CurseForge file by numeric ID.
//!
//! ### How to find a `mod_id` / `file_id`
//!
//! - **`mod_id`** is shown as "Project ID" in the About sidebar of
//!   `https://www.curseforge.com/minecraft/mc-mods/<slug>` (e.g. JEI = `238222`).
//! - **`file_id`** is the trailing segment of a file URL:
//!   `https://www.curseforge.com/minecraft/mc-mods/<slug>/files/<file_id>`.
//!
//! Unlike Modrinth, the CurseForge API requires an API key. Configure
//! it once before `.run()` via
//! [`lighty_launcher::mods::curseforge::set_api_key`].
//! Get a key at <https://console.curseforge.com/?#/api-keys>.
//!
//! `VersionBuilder::new(name, Loader::Fabric, loader_version, mc_version)`.
//!
//! - MC versions:             <https://piston-meta.mojang.com/mc/game/version_manifest_v2.json>
//! - Fabric loaders / MC:     <https://meta.fabricmc.net/v2/versions/loader/{mc}>
//! - CurseForge project page: `https://www.curseforge.com/minecraft/mc-mods/<slug>`
//! - CurseForge API:          <https://docs.curseforge.com/>

use lighty_launcher::mods::curseforge;
use lighty_launcher::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(feature = "tracing")]
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    AppState::init("LightyLauncher")?;

    // CurseForge API key — read from env to keep the key out of source.
    // Export `CURSEFORGE_API_KEY=...` before running the example.
    curseforge::set_api_key(std::env::var("CURSEFORGE_API_KEY")?);

    // Authenticate
    let mut auth = OfflineAuth::new("Hamadi");
    #[cfg(feature = "events")]
    let profile = auth.authenticate(None).await?;
    #[cfg(not(feature = "events"))]
    let profile = auth.authenticate().await?;

    let instance =
        VersionBuilder::new("curseforge-jei-1.21.1", Loader::Fabric, "0.17.2", "1.21.1");

    instance
        .with_mod()
            .with_curseforge_mods(vec![
                // Latest file compatible with MC + loader:
                (238222, None),              // JEI — Just Enough Items
                // Pinned to a specific `file_id`. See the header-doc for
                // the procedure. Replace `5234567` with a real id from
                // https://www.curseforge.com/minecraft/mc-mods/jei/files
                // before running.
                (238222, Some(5234567)),
            ])
            .done()
        .launch(&profile, JavaDistribution::Temurin)
        .with_arguments()
            .set(KEY_GAME_DIRECTORY, "runtime") //better folder organization
            .done()
        .run()
        .await?;

    trace_info!("CurseForge launch successful!");

    Ok(())
}
