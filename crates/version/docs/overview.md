# Overview — `lighty-version`

Builders that describe a Minecraft instance: name, loader, versions,
on-disk paths, and the optional mod sources to install before launch.

The launch pipeline (`lighty-launch`) and the loader pipeline
(`lighty-loaders`) both take anything implementing `VersionInfo` and
`WithMods`. This crate provides the two canonical implementors:

| Type | Use case |
|------|----------|
| `VersionBuilder<L>` | Standard launcher: pick a loader + Minecraft version, optionally attach Modrinth / CurseForge mods or a modpack. |
| `LightyVersionBuilder` | Custom updater server (LightyUpdater): loader and Minecraft version come from a remote API; you can layer user mods on top. |

Both implement `Clone`, `Debug`, `VersionInfo`, and `WithMods`. They
own the directory layout for the instance — game dir, runtime dir,
JVM dir — derived from the global [`AppState`](../../core/docs/app_state.md)
unless overridden.

## How it fits

```
                ┌──────────────────────────────┐
                │       Your application        │
                └───────────────┬──────────────┘
                                │
                ┌───────────────▼──────────────┐
                │  VersionBuilder<L>           │
                │  LightyVersionBuilder        │
                │   (this crate)               │
                └───────────────┬──────────────┘
                                │ implements VersionInfo + WithMods
        ┌───────────────────────┼────────────────────────┐
        ▼                       ▼                        ▼
 lighty-loaders          lighty-launch            lighty-modsloader
 (metadata + merge)      (install + launch)       (mods + modpacks)
```

The builders are *plain data*. They do no I/O on their own — the
metadata fetch, the install pipeline, and the JVM spawn all happen
when you hand the builder to `lighty-launch`.

## Quickstart

```rust,no_run
use lighty_core::AppState;
use lighty_loaders::types::Loader;
use lighty_version::VersionBuilder;

AppState::init("MyLauncher").unwrap();

let instance = VersionBuilder::new(
    "fabric-1.21",
    Loader::Fabric,
    "0.17.2",
    "1.21.1",
);
```

For the full launch flow (auth, JVM, install, run) see
[`how-to-use.md`](./how-to-use.md).

## Default paths

Derived from [`AppState`](../../core/docs/app_state.md):

| Field | Default |
|-------|---------|
| `game_dirs` | `AppState::data_dir().join(name)` |
| `runtime_dir` | alias of `game_dirs` until overridden via the launch builder |
| `java_dirs` | `AppState::config_dir().join("jre")` |

Override `java_dirs` with `with_custom_java_dir(path)`. Override the
runtime dir at launch time with the `KEY_GAME_DIRECTORY` argument key
(see [`crates/launch/docs/arguments.md`](../../launch/docs/arguments.md)).

## Mod sources

Behind the `modrinth` / `curseforge` features, both builders open a
sub-builder via `.with_mod()`:

- `VersionBuilder.with_mod()` returns a `ModSourcesBuilder<L>` that
  supports individual mods *and* a single modpack source.
- `LightyVersionBuilder.with_mod()` returns a `LightyModSourcesBuilder`
  that only supports individual mods. Modpacks are not exposed because
  the LightyUpdater server is itself the modpack source of truth —
  mixing in a second one would conflict on the loader / MC version.

The clients, parsers and resolver live in
[`lighty-modsloader`](../../modsloader/docs/overview.md).

## See also

- [`how-to-use.md`](./how-to-use.md) — practical examples + full launch flow.
- [`version-builder.md`](./version-builder.md) — `VersionBuilder<L>` API details.
- [`lighty-version-builder.md`](./lighty-version-builder.md) — `LightyVersionBuilder` API details.
- [`exports.md`](./exports.md) — public surface listing.
- [`crates/loaders/docs/traits.md`](../../loaders/docs/traits.md) — `VersionInfo` and `LoaderExtensions`.
