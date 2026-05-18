# Exports

Public surface of `lighty-loaders`.

## Root re-exports

```rust
pub use lighty_loaders::{
    // Types
    Loader, LoaderExtensions, VersionInfo, version_metadata,

    // Loader modules (feature-gated)
    fabric,         // feature = "fabric"
    forge,          // feature = "forge"  (covers legacy + modern)
    lighty_updater, // feature = "lighty_updater"
    neoforge,       // feature = "neoforge"
    optifine,       // always on (depends on vanilla)
    quilt,          // feature = "quilt"
    vanilla,        // feature = "vanilla"

    // Utils
    cache, manifest, query,
};
```

## `types`

```rust
// Re-exported flat
pub use lighty_loaders::types::{Loader, LoaderExtensions, VersionInfo, version_metadata};

// Also under types::
pub use lighty_loaders::types::{InstanceSize, version_metadata};
```

### `Loader` enum

```rust
pub enum Loader {
    Fabric,
    NeoForge,
    Optifine,
    Quilt,
    Vanilla,
    Forge,
    LightyUpdater,
}
```

### `version_metadata` module

Re-exported types (see `crates/loaders/src/types/version/version_metadata/`):

```rust
pub use lighty_loaders::version_metadata::{
    Version, VersionMetaData,
    Library, Arguments, JavaVersion, AssetIndex, Downloads,
    Mods, Native, MainClass,
    // …and the wire types for the underlying Mojang manifest
};
```

The exact list mirrors what the launch crate needs; check
`src/types/version/version_metadata/` for the live API.

## Loader modules

Each loader module exposes:

- A query enum (e.g. `vanilla::VanillaQuery`, `fabric::FabricQuery`).
- A `Lazy<ManifestRepository<…>>` singleton (e.g.
  `vanilla::VANILLA`, `fabric::FABRIC`, `forge::FORGE`).
- Wire types specific to that loader.

Forge also exposes installer helpers:

```rust
pub use lighty_loaders::forge::forge::{
    FORGE,
    ForgeQuery, ForgeRawData,
    extract_install_profile_libraries_modern,
    build_installer_url, installer_cache_path,
};
pub use lighty_loaders::forge::forge_legacy::{
    is_legacy_forge, legacy_installer_path, InstallProfileKind,
};
```

## `utils`

```rust
pub use lighty_loaders::utils::{cache, manifest, query};
```

### `utils::query`

```rust
pub trait Query { /* see query.md */ }
pub struct QueryKey<Q> { pub version: String, pub query: Q }
pub type Result<T> = std::result::Result<T, lighty_core::QueryError>;
```

### `utils::manifest`

```rust
pub struct ManifestRepository<F: Query> { /* private */ }

impl<F: Query> ManifestRepository<F> {
    pub fn new() -> Self;
    pub async fn get<V: VersionInfo>(&self, version: &V, query: F::Query) -> Result<Arc<F::Data>>;
    pub async fn get_raw<V: VersionInfo>(&self, version: &V) -> Result<Arc<<F as Query>::Raw>>;
}
```

### `utils::cache`

```rust
pub struct Cache<K, V> { /* private */ }

impl<K, V> Cache<K, V> where K: Eq + Hash + Clone + Send + Sync + 'static,
                              V: Clone + Send + Sync + 'static {
    pub fn new() -> Self;
    pub fn with_smart_cleanup() -> Self;
    pub async fn get_or_try_insert_with<F, Fut>(&self, key: K, ttl: Duration, init: F)
        -> Result<V, QueryError>
    where F: FnOnce() -> Fut, Fut: Future<Output = Result<V, QueryError>>;
}
```

## Errors

All loader operations return `Result<_, lighty_core::QueryError>`.
`QueryError` is not re-exported by this crate — use the full path or
the `lighty_core` re-export. Variants:
[`../../core/docs/exports.md`](../../core/docs/exports.md#errors).

## Cargo features

```toml
[features]
vanilla         = []
fabric          = ["vanilla"]
quilt           = ["vanilla"]
neoforge        = ["vanilla"]
forge           = ["vanilla"]      # legacy + modern in one flag
lighty_updater  = ["vanilla", "fabric", "quilt", "neoforge", "forge"]
modrinth        = ["vanilla"]      # enables mod-source clients (modsloader)
curseforge      = ["vanilla"]
all-mods        = ["modrinth", "curseforge"]
all-loaders     = ["vanilla", "fabric", "quilt", "neoforge", "forge",
                   "lighty_updater", "all-mods"]
```

## See also

- [`how-to-use.md`](./how-to-use.md) — common usage pattern
- [`traits.md`](./traits.md) — `VersionInfo` + `LoaderExtensions`
- [`query.md`](./query.md) — `Query` trait + `ManifestRepository`
- [`cache.md`](./cache.md) — TTL + cleanup
- [`events.md`](./events.md) — `LoaderEvent` variants
- Per-loader pages: [`loaders/`](./loaders/)
