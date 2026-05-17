# lighty-modsloader

User-attached mods + modpacks for `lighty-launcher`. Lives next to
`lighty-loaders` in the workspace; it deals with **what to install on
top of a vanilla/Fabric/Forge/NeoForge/Quilt instance**, not with the
loaders themselves.

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

There is no separate `modpack` feature — enabling either provider
activates its modpack format parser (which lives **inside** the
provider's module, not as a sibling). The top-level `lighty-launcher`
crate forwards these as `modrinth`, `curseforge`, `all-mods` features.

## Where the pipeline lives

```
crates/
├── modsloader/                       this crate
│   ├── request.rs                    ModRequest / ModSource / ModKey
│   ├── with_mods.rs                  WithMods trait (implemented by VersionBuilder)
│   ├── instance_cache.rs             InstanceCache helper trait
│   ├── resolver.rs                   BFS over user requests + transitive deps
│   ├── modpack.rs                    flat file — ModpackSource enum + From impls
│   ├── modrinth/                     Modrinth provider
│   │   ├── api.rs                    BASE_URL, USER_AGENT, PROVIDER, url_encode
│   │   ├── client.rs                 fetch + MODRINTH_CACHE + PROJECT_TYPE_CACHE
│   │   ├── client_metadata.rs        Labrinth wire types (ModrinthVersion, …)
│   │   ├── modpack.rs                .mrpack URL resolver + manifest parser
│   │   └── modpack_metadata.rs       .mrpack wire types (MrpackManifest, …)
│   └── curseforge/                   CurseForge provider
│       ├── api.rs                    BASE_URL, PROVIDER, set_api_key, read_api_key, url_encode
│       ├── client.rs                 fetch, fetch_pinned_file, install_subdir_for
│       ├── client_metadata.rs        Core-API wire types + CLASS_* / MOD_LOADER_* / DEP_*
│       ├── modpack.rs                CurseForge .zip URL resolver + manifest parser
│       └── modpack_metadata.rs       CF modpack wire types (CfModpackManifest, …)
└── launch/
    └── src/installer/ressources/
        ├── mods.rs                   consumes Mods.path verbatim (qualified by sub-folder)
        └── modpack/                  downloads archive, extracts, merges overrides
```

The split is intentional:

- **Parsing / API clients / asset-routing** are pure functions / async
  fetches — they belong in modsloader.
- **Download + extract + overrides + cache idempotence** touch the
  runtime directory and emit events — they belong in
  `lighty-launch::installer::ressources::modpack`.

## Trait bridge: `WithMods`

`lighty-launch` requires `T: VersionInfo + LoaderExtensions + Arguments
+ Installer + WithMods` to spin up its launch pipeline. The `WithMods`
default returns `&[]`, so vanilla instances that never call
`.with_mod()` pay nothing.

```rust
pub trait WithMods {
    fn mod_requests(&self) -> &[ModRequest];

    #[cfg(any(feature = "modrinth", feature = "curseforge"))]
    fn modpack(&self) -> Option<&ModpackSource> { None }
}
```

## See also

- [`mods.md`](./mods.md) — pinning Modrinth `version_id` / CurseForge
  `file_id`, the asset-routing contract (mods / resourcepacks /
  shaderpacks / datapacks), how the BFS resolver walks `required` deps.
- [`modpacks.md`](./modpacks.md) — format of `.mrpack` and CurseForge
  `.zip`, conflict policy for overrides.
- [`events.md`](./events.md) — the eleven `ModloaderEvent` variants.
- [`exports.md`](./exports.md) — public types exported by the crate.
- [`../../launch/docs/installation.md`](../../launch/docs/installation.md)
  — the launch-side pipeline that consumes `WithMods`.
- [`../../../ASSETS_ROUTING.md`](../../../ASSETS_ROUTING.md) — design
  doc for the asset-kind routing refactor.
