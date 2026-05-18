# Exports

Public surface of `lighty-modsloader`. The top-level
`lighty_launcher::mods` re-exports the whole crate.

## Always available

```rust
pub use lighty_modsloader::{
    ModRequest,    // user-facing mod request (Modrinth | CurseForge)
    ModSource,     // tag enum (Modrinth, CurseForge)
    ModKey,        // dedup key — (source, project id), version-agnostic
    WithMods,      // builder bridge trait — implemented by VersionBuilder
    InstanceCache, // per-instance cache helpers (clear_cache, …)
};
```

### `ModRequest`

```rust
pub enum ModRequest {
    Modrinth   { id_or_slug: String, version: Option<String> },
    CurseForge { mod_id: u32,        file_id: Option<u32> },
}
```

### `WithMods`

```rust
pub trait WithMods {
    fn mod_requests(&self) -> &[ModRequest];

    #[cfg(any(feature = "modrinth", feature = "curseforge"))]
    fn modpack(&self) -> Option<&ModpackSource> { None }
}
```

## Feature-gated

### `modrinth` or `curseforge` (resolver + modpack source)

```rust
pub use lighty_modsloader::resolver::resolve;
pub use lighty_modsloader::ModpackSource;          // re-exported at crate root
// ModpackSource also reachable via `lighty_modsloader::modpack::ModpackSource`
// (the `modpack` module is now a flat file containing the enum + From impls).
```

### `modrinth`

```rust
// Module — `lighty_modsloader::modrinth`
pub use lighty_modsloader::modrinth::{
    fetch,                                          // ModRequest → (Mods, Vec<ModRequest>)
    ModrinthKey,                                    // cache key
    MODRINTH_CACHE,                                 // process-wide Cache
};

// Submodules (use directly when you need wire types or helpers):
pub use lighty_modsloader::modrinth::api;           // BASE_URL, USER_AGENT, PROVIDER, url_encode
pub use lighty_modsloader::modrinth::client;        // fetch, MODRINTH_CACHE, ModrinthKey
pub use lighty_modsloader::modrinth::client_metadata::{
    ModrinthProject, ModrinthVersion, ModrinthFile,
    ModrinthHashes, ModrinthDependency,
};
pub use lighty_modsloader::modrinth::modpack::{
    resolve_mrpack_url, parse_manifest,
};
pub use lighty_modsloader::modrinth::modpack_metadata::{
    MrpackManifest, MrpackFile, MrpackHashes, MrpackEnv, MrpackDependencies,
};
```

### `curseforge`

```rust
// Module — `lighty_modsloader::curseforge`
pub use lighty_modsloader::curseforge::{
    fetch,                                          // ModRequest → (Mods, Vec<ModRequest>)
    fetch_pinned_file,                              // used by the modpack pipeline
    set_api_key,                                    // re-exported from api::set_api_key
    CurseForgeKey,                                  // cache key
    CURSEFORGE_CACHE,                               // process-wide Cache
};

// Submodules:
pub use lighty_modsloader::curseforge::api;         // BASE_URL, PROVIDER, set_api_key,
                                                     // read_api_key, url_encode (+ API_KEY storage)
pub use lighty_modsloader::curseforge::client::{
    fetch, fetch_pinned_file, install_subdir_for,   // pub helper for the modpack pipeline
    CurseForgeKey, CURSEFORGE_CACHE,
};
pub use lighty_modsloader::curseforge::client_metadata::{
    // wire types
    CurseForgeEnvelope, CurseForgeMod, CurseForgeFile,
    CurseForgeHash, CurseForgeDependency,
    // constants
    CLASS_MOD, CLASS_RESOURCE_PACK, CLASS_SHADER_PACK, CLASS_MODPACK,
    MOD_LOADER_FORGE, MOD_LOADER_FABRIC, MOD_LOADER_QUILT, MOD_LOADER_NEOFORGE,
    HASH_ALGO_SHA1, HASH_ALGO_MD5,
    DEP_EMBEDDED_LIBRARY, DEP_OPTIONAL, DEP_REQUIRED, DEP_TOOL,
    DEP_INCOMPATIBLE, DEP_INCLUDE,
};
pub use lighty_modsloader::curseforge::modpack::{
    resolve_cf_modpack_url, parse_manifest,
};
pub use lighty_modsloader::curseforge::modpack_metadata::{
    CfModpackManifest, CfMinecraftSpec, CfModLoader, CfModpackFile,
};
```

## Error types

All errors flow through `lighty_core::QueryError` (shared with loaders
and launch). Modsloader doesn't define its own error enum. The new
asset-routing surface adds `QueryError::UnsupportedFormat { what,
expected, found }` as a possible outcome of `fetch` when the project
type / classId isn't routable.

## Conversions

`From<&str> for ModpackSource` and `From<String> for ModpackSource`
both produce `ModpackSource::ModrinthUrl` — the most common case
(paste a CDN URL).
