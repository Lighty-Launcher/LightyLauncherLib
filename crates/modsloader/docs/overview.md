# lighty-modsloader

Mods + modpacks for LightyLauncher. Sits next to
[`lighty-loaders`](../../loaders/docs/overview.md): the loaders crate
resolves how the **game** runs (vanilla / Fabric / Forge / …), this
crate resolves **what to install on top** — individual mods,
resourcepacks, shaderpacks, datapacks, full modpacks.

## What it provides

| Area | Module | Feature gate |
|---|---|---|
| Public request types (`ModRequest`, `ModSource`, `ModKey`) | `request` | always on |
| Builder bridge trait (`WithMods`) | `with_mods` | always on |
| Per-instance cache helpers (`InstanceCache`) | `instance_cache` | always on |
| Source-agnostic BFS resolver (native `ModloaderEvent::Resolve*` emission with `events`) | `resolver` | `modrinth` or `curseforge` |
| `ModpackSource` enum + `From<&str>` / `From<String>` impls | `modpack` (flat file) | `modrinth` or `curseforge` |
| Modrinth Labrinth-API client + `.mrpack` parser | `modrinth` | `modrinth` |
| CurseForge Core-API client + `.zip` parser | `curseforge` | `curseforge` |

## Cargo features

```toml
[dependencies]
lighty-modsloader = { version = "...", default-features = false, features = [
    "events",        # native ModloaderEvent emission on a user-provided EventBus
    "modrinth",      # Modrinth API client + resolver + .mrpack parsing
    "curseforge",    # CurseForge API client + resolver + .zip parsing
    "all-mods",      # = modrinth + curseforge
    "tracing",       # forward tracing macros (cooperates with lighty-core)
] }
```

There's no separate `modpack` feature — enabling either provider
activates its modpack format parser. The top-level `lighty-launcher`
crate forwards these as `modrinth`, `curseforge`, `all-mods`.

## File layout

```
crates/modsloader/src/
├── request.rs               ModRequest / ModSource / ModKey
├── with_mods.rs             WithMods trait (implemented by VersionBuilder)
├── instance_cache.rs        InstanceCache helper trait
├── resolver.rs              BFS over user requests + transitive deps
├── modpack.rs               flat file — ModpackSource enum + From impls
├── modrinth/                Modrinth provider
│   ├── api.rs               BASE_URL, USER_AGENT, PROVIDER, url_encode
│   ├── client.rs            fetch + MODRINTH_CACHE + PROJECT_TYPE_CACHE
│   ├── client_metadata.rs   Labrinth wire types
│   ├── modpack.rs           .mrpack URL resolver + manifest parser
│   └── modpack_metadata.rs  .mrpack wire types
└── curseforge/              CurseForge provider
    ├── api.rs               BASE_URL, PROVIDER, set_api_key, read_api_key, url_encode
    ├── client.rs            fetch, fetch_pinned_file, install_subdir_for
    ├── client_metadata.rs   Core-API wire types + constants
    ├── modpack.rs           CF .zip URL resolver + manifest parser
    └── modpack_metadata.rs  CF modpack wire types
```

The split with the launch crate is intentional:

- **Parsing / API clients / asset routing** are pure functions /
  async fetches — they belong here.
- **Download + extract + overrides + cache idempotence** touch the
  runtime directory and emit events — they live in
  `crates/launch/src/installer/ressources/modpack/`.

## Bridge trait

`lighty-launch` requires `T: VersionInfo + LoaderExtensions +
Arguments + Installer + WithMods` to build its launch pipeline.
`WithMods` has a `&[]` default, so a vanilla instance that never
calls `.with_mod()` pays nothing.

```rust
pub trait WithMods {
    fn mod_requests(&self) -> &[ModRequest];

    #[cfg(any(feature = "modrinth", feature = "curseforge"))]
    fn modpack(&self) -> Option<&ModpackSource> { None }
}
```

## See also

- [`how-to-use.md`](./how-to-use.md) — wiring `.with_mod()` to a builder
- [`mods.md`](./mods.md) — pinning, asset routing, dependency BFS
- [`modpacks.md`](./modpacks.md) — `.mrpack` / CurseForge `.zip` format,
  conflict policy
- [`events.md`](./events.md) — `ModloaderEvent` variants
- [`exports.md`](./exports.md) — public API surface
- [`../../launch/docs/installation.md`](../../launch/docs/installation.md)
  — what the launch pipeline does with the resolved request list
