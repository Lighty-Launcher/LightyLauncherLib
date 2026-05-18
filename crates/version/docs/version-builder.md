# `VersionBuilder<L>`

Standard instance descriptor: pick a loader, a Minecraft version, a
loader version, and (optionally) mods or a modpack. The generic
`L` parameter is the loader tag — almost always `Loader` from
`lighty-loaders`. The default `L = ()` is mostly for tests.

For the broader launch story (auth + Java + install + spawn) see
[`how-to-use.md`](./how-to-use.md). For the design pitch see
[`overview.md`](./overview.md).

## Fields

| Field | Type | Notes |
|-------|------|-------|
| `name` | `String` | Used as the on-disk instance folder and in event payloads. |
| `loader` | `L` | Loader tag (`Loader::Fabric`, `Loader::Forge`, ...). |
| `loader_version` | `String` | Empty for `Loader::Vanilla`. |
| `minecraft_version` | `String` | e.g. `"1.21.1"`. |
| `game_dirs` | `PathBuf` | Default: `AppState::data_dir().join(name)`. |
| `runtime_dir` | `PathBuf` | Default: alias of `game_dirs`. Relocated via launch arg `KEY_GAME_DIRECTORY`. |
| `java_dirs` | `PathBuf` | Default: `AppState::config_dir().join("jre")`. |
| `mod_requests` | `Vec<ModRequest>` | Filled via `.with_mod()` sub-builder. |
| `modpack` | `Option<ModpackSource>` | Modrinth `.mrpack` or CurseForge `.zip` (feature-gated). |
| `ttl_override` | `Option<Duration>` | Per-instance manifest cache TTL. Default 24 h. |

## Constructor

```rust
use lighty_core::AppState;
use lighty_loaders::types::Loader;
use lighty_version::VersionBuilder;

AppState::init("MyLauncher").unwrap();

let instance = VersionBuilder::new(
    "fabric-1.21",        // on-disk folder name
    Loader::Fabric,       // loader tag
    "0.17.2",             // loader version (empty for Vanilla)
    "1.21.1",             // Minecraft version
);
```

Panics if [`AppState::init`](../../core/docs/app_state.md) wasn't called.

## Builder methods

| Method | Effect |
|--------|--------|
| `with_loader(loader)` | Replace the loader tag. |
| `with_loader_version(v)` | Replace the loader version string. |
| `with_minecraft_version(v)` | Replace the Minecraft version. |
| `with_custom_java_dir(path)` | Override the JRE install dir (default `AppState::config_dir()/jre`). |
| `with_ttl_duration(d)` | Per-instance manifest cache TTL. Default 24 h. |
| `with_mod()` | Open the mod-sources sub-builder (`ModSourcesBuilder<L>`). |

## Mod sources sub-builder

`with_mod()` returns a `ModSourcesBuilder<L>`. Methods are gated on
the `modrinth` / `curseforge` features.

```rust
# use lighty_core::AppState;
# use lighty_loaders::types::Loader;
# use lighty_version::VersionBuilder;
# AppState::init("MyLauncher").unwrap();
let instance = VersionBuilder::new("modded", Loader::Fabric, "0.17.2", "1.21.1")
    .with_mod()
        .with_modrinth_mods(vec![
            ("sodium", None),                              // latest compatible
            ("lithium", Some("mc1.21.1-0.13.0".into())),   // pinned version_id
        ])
        .done();
```

| Method | Feature | Description |
|--------|---------|-------------|
| `with_modrinth_mods(Vec<(slug_or_id, Option<version_id>)>)` | `modrinth` | Latest-compatible or pinned per entry. |
| `with_curseforge_mods(Vec<(mod_id, Option<file_id>)>)` | `curseforge` | Numeric ids only. Requires `lighty_modsloader::curseforge::set_api_key`. |
| `with_modrinth_modpack(impl Into<ModpackSource>)` | `modrinth` | Accepts a CDN URL or `ModpackSource::ModrinthPinned`. |
| `with_curseforge_modpack(project_id, file_id)` | `curseforge` | Numeric `(project_id, file_id)`. |
| `done()` | — | Returns the parent `VersionBuilder<L>` with the mods/modpack threaded in. |

Mod source semantics, classification (mods vs resourcepacks vs
shaderpacks vs datapacks), and resolver behaviour live in
[`crates/modsloader/docs/mods.md`](../../modsloader/docs/mods.md) and
[`crates/modsloader/docs/modpacks.md`](../../modsloader/docs/modpacks.md).

## Implements

- `VersionInfo` (and `&VersionBuilder<L>: VersionInfo` for read-only call sites).
- `WithMods` (and on the shared reference).
- `Clone`, `Debug`.

Through `VersionInfo`, every `LoaderExtensions` method becomes
available — `get_metadata`, `get_libraries`, `get_assets`, etc. See
[`crates/loaders/docs/traits.md`](../../loaders/docs/traits.md).

## Typical launch

```rust
use lighty_core::AppState;
use lighty_loaders::types::Loader;
use lighty_version::VersionBuilder;
use lighty_auth::{offline::OfflineAuth, Authenticator};
use lighty_java::JavaDistribution;
use lighty_launch::Launch;

# async fn run() -> anyhow::Result<()> {
AppState::init("MyLauncher")?;

let mut auth = OfflineAuth::new("Player");
let profile = auth.authenticate().await?;

let mut instance = VersionBuilder::new("fabric", Loader::Fabric, "0.17.2", "1.21.1");

instance.launch(&profile, JavaDistribution::Temurin)
    .with_jvm_options()
        .set("Xmx", "4G")
        .done()
    .run()
    .await?;
# Ok(()) }
```

## See also

- [`how-to-use.md`](./how-to-use.md) — broader launch walkthroughs.
- [`lighty-version-builder.md`](./lighty-version-builder.md) — sibling type for LightyUpdater servers.
- [`exports.md`](./exports.md) — full export listing.
- [`crates/launch/docs/launch.md`](../../launch/docs/launch.md) — the launch pipeline.
