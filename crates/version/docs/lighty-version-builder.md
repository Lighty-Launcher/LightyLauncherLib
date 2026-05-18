# `LightyVersionBuilder`

Instance descriptor for a [LightyUpdater](https://github.com/Lighty-Launcher/LightyUpdater)
server — a custom HTTP API that publishes a modpack manifest
(loader, Minecraft version, mod list with SHA-1) which the launcher
fetches at install time.

Differences vs. [`VersionBuilder`](./version-builder.md):

| Aspect | `VersionBuilder<L>` | `LightyVersionBuilder` |
|--------|---------------------|------------------------|
| Loader pin | Required at construction | Discovered from the server response |
| Minecraft version pin | Required at construction | Discovered from the server response |
| Modpack support | Yes (Modrinth `.mrpack` or CurseForge `.zip`) | **No** — the server is itself the modpack source |
| User mods on top | Yes | Yes (Modrinth / CurseForge), layered over the server-pushed list |

## Fields

| Field | Type | Notes |
|-------|------|-------|
| `name` | `String` | On-disk instance folder. |
| `server_url` | `String` | LightyUpdater endpoint. Returned by `loader_version()` to keep `VersionInfo` uniform. |
| `minecraft_version` | `Option<String>` | Filled after the first metadata fetch. |
| `loader` | `Option<Loader>` | Filled after the first metadata fetch. |
| `game_dirs` | `PathBuf` | Default: `AppState::data_dir().join(name)`. |
| `java_dirs` | `PathBuf` | Default: `AppState::config_dir().join("jre")`. |
| `mod_requests` | `Vec<ModRequest>` | User-attached mods, layered on top of the server list. |

## Constructor

```rust
use lighty_core::AppState;
use lighty_version::LightyVersionBuilder;

AppState::init("MyLauncher").unwrap();

let instance = LightyVersionBuilder::new(
    "my-server",                          // on-disk folder name
    "https://updater.example.com",        // LightyUpdater endpoint
);
```

Panics if [`AppState::init`](../../core/docs/app_state.md) wasn't called.

## Builder methods

| Method | Effect |
|--------|--------|
| `with_mod()` | Open the user-mods sub-builder (`LightyModSourcesBuilder`). |

There are no fluent setters for `loader` / `minecraft_version` /
`server_url` after construction — they're either fixed (`name`,
`server_url`) or come from the server (`loader`,
`minecraft_version`).

## User mods sub-builder

`with_mod()` returns a `LightyModSourcesBuilder`. The two methods are
feature-gated.

```rust
# use lighty_core::AppState;
# use lighty_version::LightyVersionBuilder;
# AppState::init("MyLauncher").unwrap();
let instance = LightyVersionBuilder::new("server", "https://updater.example.com")
    .with_mod()
        .with_modrinth_mods(vec![
            ("sodium-extra", None),
            ("iris", None),
        ])
        .with_curseforge_mods(vec![(238222, None)])  // JEI
        .done();
```

| Method | Feature | Description |
|--------|---------|-------------|
| `with_modrinth_mods(Vec<(slug_or_id, Option<version_id>)>)` | `modrinth` | Layered on top of the server list. |
| `with_curseforge_mods(Vec<(mod_id, Option<file_id>)>)` | `curseforge` | Requires `lighty_modsloader::curseforge::set_api_key`. |
| `done()` | — | Returns the parent `LightyVersionBuilder` with the user mods appended to `mod_requests`. |

No `with_*_modpack(...)` is exposed on purpose — see the struct doc
for the rationale (loader/MC conflict).

## Implements

- `VersionInfo` (and on `&LightyVersionBuilder`). `loader()` returns
  `Loader::LightyUpdater` until the server response has been fetched.
- `WithMods` (and on the shared reference). `modpack()` always
  returns `None` here.
- `Clone`, `Debug`.

## Typical launch

```rust
use lighty_core::AppState;
use lighty_version::LightyVersionBuilder;
use lighty_auth::{microsoft::MicrosoftAuth, Authenticator};
use lighty_java::JavaDistribution;
use lighty_launch::Launch;

# async fn run() -> anyhow::Result<()> {
AppState::init("MyLauncher")?;

let mut auth = MicrosoftAuth::new("your-azure-client-id");
let profile = auth.authenticate().await?;

let mut instance = LightyVersionBuilder::new(
    "survival",
    "https://play.myserver.com/api",
);

instance.launch(&profile, JavaDistribution::Temurin)
    .run()
    .await?;
# Ok(()) }
```

The launch pipeline calls the server, fetches the manifest, populates
`loader` / `minecraft_version`, downloads the listed mods (verifying
SHA-1), then proceeds like any other instance.

## Server contract (summary)

`GET {server_url}` returns:

```json
{
  "minecraft_version": "1.21.1",
  "loader": "Fabric",
  "loader_version": "0.17.2",
  "mods": [
    { "name": "...", "url": "...", "sha1": "...", "enabled": true }
  ]
}
```

`loader` accepts `"vanilla" | "fabric" | "quilt" | "neoforge" | "forge"`.
See the [LightyUpdater repository](https://github.com/Lighty-Launcher/LightyUpdater)
for the full spec and a reference server.

## See also

- [`how-to-use.md`](./how-to-use.md) — practical walkthroughs.
- [`version-builder.md`](./version-builder.md) — the standard builder.
- [`exports.md`](./exports.md) — full export listing.
- [`crates/modsloader/docs/mods.md`](../../modsloader/docs/mods.md) — mod request semantics.
