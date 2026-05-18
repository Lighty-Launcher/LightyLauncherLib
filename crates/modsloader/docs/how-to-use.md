# Using lighty-modsloader

You usually don't talk to `lighty-modsloader` directly — its `WithMods`
trait is implemented for `VersionBuilder`, and the launch crate
chains everything from `.with_mod()`. This page shows the common
patterns end-to-end.

## Add a few mods to an instance

```rust,no_run
use lighty_core::AppState;
use lighty_loaders::Loader;
use lighty_version::VersionBuilder;
use lighty_modsloader::WithMods;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("LightyLauncher")?;

    let v = VersionBuilder::new("fabric-1.21", Loader::Fabric, "0.16.9", "1.21.1");

    let v = v
        .with_mod()
            .with_modrinth_mods(vec![
                ("sodium",       None),   // mod
                ("sodium-extra", None),   // resourcepack — routed automatically
            ])
            .with_curseforge_mods(vec![
                (238222, None),           // JEI
            ])
            .done();

    println!("queued {} requests", v.mod_requests().len());
    // hand `v` to lighty-launch.launch(...).run().await
    Ok(())
}
```

Per-source detail: [`mods.md`](./mods.md).

## Add a modpack

```rust,no_run
use lighty_core::AppState;
use lighty_loaders::Loader;
use lighty_version::VersionBuilder;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("LightyLauncher")?;

    let v = VersionBuilder::new("pack", Loader::Fabric, "0.16.9", "1.21.1");

    // Modrinth — paste a CDN URL
    let v = v
        .with_mod()
            .with_modrinth_modpack("https://cdn.modrinth.com/data/.../pack.mrpack")
            .done();
    let _ = v;
    Ok(())
}
```

CurseForge variant takes `(project_id, file_id)`. Format details +
override conflict policy in [`modpacks.md`](./modpacks.md).

## CurseForge API key

CurseForge requires an API key. Set it once before any launch:

```rust,no_run
use std::env;

# fn run() -> anyhow::Result<()> {
lighty_modsloader::curseforge::set_api_key(env::var("CURSEFORGE_API_KEY")?);
# Ok(()) }
```

Get a key at <https://console.curseforge.com/?#/api-keys>.

## Subscribe to resolver / install events

With the `events` feature on, the resolver and the launch-side
modpack pipeline emit `ModloaderEvent`:

```rust,no_run
use lighty_event::{Event, EventBus, ModloaderEvent};

let bus = EventBus::new(1000);
let mut rx = bus.subscribe();
tokio::spawn(async move {
    while let Ok(event) = rx.next().await {
        if let Event::Modloader(me) = event {
            match me {
                ModloaderEvent::ResolveStarted { request_count } =>
                    println!("[resolve] {request_count} requests"),
                ModloaderEvent::ModpackInstalled { name, mods_count } =>
                    println!("[pack]    {name}: {mods_count} mods"),
                ModloaderEvent::ResourcePacksInstalled { count, bytes } =>
                    println!("[rp]      {count} files / {bytes} B"),
                _ => {}
            }
        }
    }
});

// .launch(...).with_event_bus(&bus).run().await?
```

Full event list: [`events.md`](./events.md).

## Mod loader compatibility

| Loader | Modrinth | CurseForge |
|---|---|---|
| Fabric, Forge, NeoForge, Quilt | OK | OK |
| Vanilla / OptiFine / LightyUpdater | rejected (`QueryError::UnsupportedLoader`) | rejected |

LightyUpdater needs to ship its own mods through the
`LightyVersionBuilder` payload — the per-mod resolver is loader-
specific by design (it filters by `(mc, loader)`).

## Errors at a glance

```rust,ignore
QueryError::ModNotFound { provider, id }
QueryError::ModIncompatible { provider, id, mc, loader }
QueryError::ModDistributionForbidden { id }                // CurseForge download_url=null
QueryError::UnsupportedFormat { what, expected, found }    // unknown project_type / classId
QueryError::UnsupportedLoader(String)                       // Vanilla / OptiFine / LightyUpdater
QueryError::Network(reqwest::Error)
```

Full enum:
[`../../core/docs/exports.md`](../../core/docs/exports.md#errors).

## See also

- [`mods.md`](./mods.md) — pinning, asset routing, dependency BFS
- [`modpacks.md`](./modpacks.md) — `.mrpack` / CurseForge `.zip`
- [`events.md`](./events.md) — `ModloaderEvent` variants
- [`exports.md`](./exports.md) — public API surface
- [`../../launch/docs/installation.md`](../../launch/docs/installation.md)
  — what happens after `.run()`
