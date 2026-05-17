//! Modrinth mod fetching example (Fabric on MC 1.21.1).
//!
//! Chain `.with_mod().with_modrinth_mods(...)` on the `VersionBuilder`
//! to pull a list of mods from Modrinth. The tuple is `(slug, version_id?)`:
//!
//! - `None` resolves the latest release compatible with the instance's
//!   MC + loader. Required dependencies are walked transitively.
//! - `Some(version_id)` pins a specific Modrinth release. **`version_id`
//!   is an opaque string** (e.g. `"PpRTuoEh"`), NOT the human-readable
//!   version number `"0.5.8"`.
//!
//! ### How to find a `version_id`
//!
//! 1. Open `https://modrinth.com/mod/<slug>/versions`.
//! 2. Click the target version.
//! 3. The URL becomes `.../mod/<slug>/version/<version_id>` — the trailing
//!    segment is the value to plug into `Some(...)`.
//!
//! `VersionBuilder::new(name, Loader::Fabric, loader_version, mc_version)`.
//!
//! - MC versions:           <https://piston-meta.mojang.com/mc/game/version_manifest_v2.json>
//! - Fabric loaders / MC:   <https://meta.fabricmc.net/v2/versions/loader/{mc}>
//! - Modrinth project page: `https://modrinth.com/mod/{slug}`
//! - Modrinth API:          <https://api.modrinth.com/v2/project/{slug}/version>

use lighty_launcher::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(feature = "tracing")]
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    AppState::init("LightyLauncher")?;

    // Authenticate
    let mut auth = OfflineAuth::new("Hamadi");
    #[cfg(feature = "events")]
    let profile = auth.authenticate(None).await?;
    #[cfg(not(feature = "events"))]
    let profile = auth.authenticate().await?;

    let instance =
        VersionBuilder::new("modrinth-cobblemon-1.21.1", Loader::Fabric, "0.17.2", "1.21.1");

    instance
        .with_mod()
            .with_modrinth_mods(vec![
                // Latest version compatible with MC + loader:
                ("cobblemon", None),
                // Pinned to a specific Modrinth `version_id`. See the
                // header-doc for the procedure to find it. The value
                // below is illustrative — replace with a real id from
                // https://modrinth.com/mod/sodium/versions before running.
                ("sodium", Some("PpRTuoEh".into())),
            ])
            .done()
        .launch(&profile, JavaDistribution::Temurin)
        .with_arguments()
            .set(KEY_GAME_DIRECTORY, "runtime") //better folder organization
            .done()
        .run()
        .await?;

    trace_info!("Modrinth launch successful!");

    Ok(())
}
