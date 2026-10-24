# Traits — VersionInfo and LoaderExtensions

The two traits that make `lighty-loaders` extensible. `VersionInfo`
describes an installable instance; `LoaderExtensions` is the
auto-implemented async fetch API on top of it.

## `VersionInfo`

```rust
pub trait VersionInfo: Clone + Send + Sync {
    type LoaderType: Clone + Send + Sync + std::fmt::Debug;

    fn name(&self)              -> &str;
    fn loader_version(&self)    -> &str;
    fn minecraft_version(&self) -> &str;
    fn game_dirs(&self)         -> &Path;        // instance root
    fn java_dirs(&self)         -> &Path;        // JRE cache
    fn loader(&self)            -> &Self::LoaderType;

    fn runtime_dir(&self)       -> &Path  { self.game_dirs() }      // ${game_directory}
    fn set_runtime_dir(&mut self, _path: PathBuf) {}                 // launch runner hook
    fn game_dir_exists(&self)   -> bool   { self.game_dirs().exists() }
    fn java_dir_exists(&self)   -> bool   { self.java_dirs().exists() }
    fn full_identifier(&self)   -> String { /* "{name}-{mc}-{loader}" */ }
    fn paths(&self)             -> (&Path, &Path) { (self.game_dirs(), self.java_dirs()) }
    fn is_installed(&self)      -> bool   { self.game_dirs().exists() }
    fn ttl(&self)               -> Duration { Duration::from_secs(86_400) }   // 24h default
}
```

The default `ttl` (24 h) feeds the cache layer — override it on
custom implementations to tighten/loosen freshness per instance.

`set_runtime_dir` is a hook the launch runner uses to write the
effective working directory back onto a mutable builder (e.g. when an
instance opens its `instance/` sub-dir).

### Who implements it

| Type | Crate | Loader type |
|---|---|---|
| `VersionBuilder` | `lighty-version` | `Loader` |
| `LightyVersionBuilder` | `lighty-version` | `LightyLoader` (server-tagged) |
| your custom type | — | your own type implementing [`LoaderKind`](#loaderkind) |

### Custom impl

```rust
use lighty_loaders::{Loader, VersionInfo};
use lighty_core::AppState;
use std::path::Path;

#[derive(Clone)]
struct MyInstance { name: String, mc: String, ver: String, loader: Loader }

impl VersionInfo for MyInstance {
    type LoaderType = Loader;

    fn name(&self)              -> &str { &self.name }
    fn loader_version(&self)    -> &str { &self.ver }
    fn minecraft_version(&self) -> &str { &self.mc }
    fn game_dirs(&self)         -> &Path { AppState::data_dir() }
    fn java_dirs(&self)         -> &Path { AppState::cache_dir() }
    fn loader(&self)            -> &Loader { &self.loader }
}
```

`LoaderExtensions` lights up automatically because of the blanket
impl below — it only requires the `LoaderType` to implement
[`LoaderKind`](#loaderkind), which `Loader` does out of the box.

## `LoaderKind`

The dispatch seam. `LoaderExtensions` knows nothing about concrete
loaders — it delegates every call to the instance's `LoaderType`
through `LoaderKind`. The built-in `Loader` enum implements it; a
consumer brings its own loader by implementing `LoaderKind` for its
own type (used as `VersionInfo::LoaderType`) **without extending the
`Loader` enum**.

```rust
#[async_trait]
pub trait LoaderKind: Clone + Send + Sync + Debug + 'static {
    /// Full metadata for an instance of this loader.
    async fn resolve_metadata<V>(&self, version: &V) -> QueryResult<Arc<VersionMetaData>>
    where V: VersionInfo<LoaderType = Self>;

    /// The coordinates the instance really installs under.
    async fn resolved_instance<V>(&self, version: &V) -> QueryResult<ResolvedInstance>
    where V: VersionInfo<LoaderType = Self>;

    /// Drops any cached manifest for the named instance.
    async fn invalidate(&self, version_name: &str);

    /// The built-in loader this kind installs *as* — drives mod
    /// resolution and the Forge/NeoForge post-install steps. A custom
    /// Fabric-based loader returns `Loader::Fabric`.
    fn base_loader(&self) -> Loader;
}
```

`resolve_metadata` is the one path a custom loader fully owns (fetch
its own manifest, build its own `Version`). `base_loader` ties the
instance back to a built-in loader so the shared install/launch
pipeline (mods, natives, Forge processors) keeps working: a loader
that is just a custom *source* over Fabric returns `Loader::Fabric`
and reuses everything else.

## `LoaderExtensions`

Auto-implemented for any `V: VersionInfo` whose `LoaderType: LoaderKind`.

```rust
#[async_trait]
pub trait LoaderExtensions {
    async fn get_metadata(&self)      -> QueryResult<Arc<VersionMetaData>>;   // full
    async fn get_libraries(&self)     -> QueryResult<Arc<VersionMetaData>>;   // base-loader
    async fn get_main_class(&self)    -> QueryResult<Arc<VersionMetaData>>;   // base-loader
    async fn get_natives(&self)       -> QueryResult<Arc<VersionMetaData>>;   // Vanilla-based
    async fn get_java_version(&self)  -> QueryResult<Arc<VersionMetaData>>;   // Vanilla-based
    async fn get_assets(&self)        -> QueryResult<Arc<VersionMetaData>>;   // Vanilla-based
    async fn invalidate_cache(&self);                                        // drop cached manifest
    async fn resolved_instance(&self) -> QueryResult<ResolvedInstance>;      // real coordinates
}
```

`get_metadata` returns the merged result; the other five return a
narrow `VersionMetaData` populated only with the requested slice (so
calling sites can avoid touching libraries when they only need
assets).

### Dispatch

`get_metadata` delegates to `self.loader().resolve_metadata(self)`.
For the built-in `Loader`, that resolves as:

| Loader | What it returns |
|---|---|
| `Vanilla` | Mojang manifest, parsed and cached. |
| `Fabric` | Vanilla + Fabric loader profile, libraries merged, main class overridden. |
| `Quilt` | Vanilla + Quilt loader profile, same merge. |
| `NeoForge` | NeoForge installer JAR, processor results, vanilla libs merged. |
| `Forge` | Forge installer JAR. Detects legacy (1.5.2 → 1.12.2) vs modern (≥ 1.13) from the install profile. |
| `LightyUpdater` | Server-defined metadata merged onto the base loader the server picks. |
| `Optifine` | Standalone (not a real loader — see [`loaders/optifine.md`](./loaders/optifine.md)). |

A `Loader` variant whose feature isn't compiled in raises
`QueryError::OperationUnsupported`. `get_libraries` / `get_main_class`
dispatch on `base_loader()` the same way; `get_natives` /
`get_java_version` / `get_assets` always hit the Vanilla manifest.

## Usage

```rust
use lighty_core::AppState;
use lighty_loaders::{Loader, LoaderExtensions};
use lighty_version::VersionBuilder;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("LightyLauncher")?;

    let v = VersionBuilder::new("fab", Loader::Fabric, "0.16.9", "1.21.1");

    let meta      = v.get_metadata().await?;       // full
    let libs_only = v.get_libraries().await?;      // narrow

    println!("full: {}, libs-only: {}",
        meta.libraries.len(), libs_only.libraries.len());
    Ok(())
}
```

## Errors

All methods return `QueryResult<_>` = `Result<_, lighty_core::QueryError>`.
The relevant variants for loaders are `Network`, `JsonParsing`,
`VersionNotFound`, `MissingField`, `UnsupportedLoader`,
`InvalidMetadata`. Full enum:
[`../../core/docs/exports.md`](../../core/docs/exports.md#errors).

## See also

- [`overview.md`](./overview.md) — loader catalogue
- [`how-to-use.md`](./how-to-use.md) — calling `get_metadata`
- [`query.md`](./query.md) — implementing a new loader
- [`cache.md`](./cache.md) — TTL behaviour (driven by `ttl()`)
- [`../../version/docs/how-to-use.md`](../../version/docs/how-to-use.md)
  — `VersionBuilder` canonical reference
