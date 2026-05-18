# Query — implementing a new loader

The `Query` trait abstracts every loader behind the same shape: fetch
a raw manifest, extract typed sub-queries from it, return a complete
`Version` when asked. `ManifestRepository<Q>` adds caching +
thundering-herd protection on top.

## Trait

```rust,ignore
#[async_trait]
pub trait Query: Send + Sync {
    type Query: Eq + Hash + Clone + Send + Sync + 'static;
    type Data:  Clone        + Send + Sync + 'static;
    type Raw:                  Send + Sync + 'static;

    fn name() -> &'static str;

    async fn fetch_full_data<V: VersionInfo>(version: &V) -> Result<Self::Raw>;

    async fn extract<V: VersionInfo>(
        version: &V,
        query:   &Self::Query,
        raw:     &Self::Raw,
    ) -> Result<Self::Data>;

    async fn version_builder<V: VersionInfo>(
        version: &V,
        full_data: &Self::Raw,
    ) -> Result<Version>;
}

pub struct QueryKey<Q> { pub version: String, pub query: Q }
```

`Result<T>` is `std::result::Result<T, lighty_core::QueryError>`.

Per-instance TTL comes from `VersionInfo::ttl()` (24 h by default) —
no per-query override is needed.

## ManifestRepository

```rust,ignore
pub struct ManifestRepository<F: Query> { /* private */ }

impl<F: Query> ManifestRepository<F> {
    pub fn new() -> Self;                                          // smart-cleanup caches

    pub async fn get<V: VersionInfo>(&self, version: &V, query: F::Query)
        -> Result<Arc<F::Data>>;

    pub async fn get_raw<V: VersionInfo>(&self, version: &V)
        -> Result<Arc<<F as Query>::Raw>>;
}
```

`get` walks: query cache → raw cache → fetch. Both caches respect the
instance's TTL and serialize concurrent fetches behind a per-key
mutex (no duplicate downloads even under contention).

## Implementing your own

Sketch — a loader that pulls a JSON manifest from a custom URL.

```rust,no_run
use async_trait::async_trait;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use lighty_core::{QueryError, hosts::HTTP_CLIENT};
use lighty_loaders::{
    Loader, VersionInfo,
    version_metadata::{Library, Version},
};
use lighty_loaders::utils::{
    manifest::ManifestRepository,
    query::Query,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MyLoaderQuery { FullMetadata, Libraries }

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyRaw {
    pub main_class: String,
    pub libraries:  Vec<MyLib>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyLib { pub name: String, pub url: String, pub sha1: String }

pub static MY_LOADER: Lazy<ManifestRepository<MyLoaderQuery>> =
    Lazy::new(ManifestRepository::new);

#[async_trait]
impl Query for MyLoaderQuery {
    type Query = MyLoaderQuery;
    type Data  = lighty_loaders::version_metadata::VersionMetaData;
    type Raw   = MyRaw;

    fn name() -> &'static str { "my_loader" }

    async fn fetch_full_data<V: VersionInfo>(version: &V) -> Result<Self::Raw, QueryError> {
        let url = format!(
            "https://my-api.example/versions/{}/{}",
            version.minecraft_version(),
            version.loader_version(),
        );

        let raw: MyRaw = HTTP_CLIENT.get(&url).send().await?
            .error_for_status()?.json().await?;
        Ok(raw)
    }

    async fn extract<V: VersionInfo>(
        version: &V,
        query: &Self::Query,
        raw: &Self::Raw,
    ) -> Result<Self::Data, QueryError> {
        match query {
            MyLoaderQuery::FullMetadata => {
                let v = Self::version_builder(version, raw).await?;
                Ok(lighty_loaders::version_metadata::VersionMetaData::Version(v))
            }
            MyLoaderQuery::Libraries => {
                // Build a VersionMetaData populated only with libraries.
                let libs = to_libraries(&raw.libraries);
                let mut v = empty_version();
                v.libraries = libs;
                Ok(lighty_loaders::version_metadata::VersionMetaData::Version(v))
            }
        }
    }

    async fn version_builder<V: VersionInfo>(
        _version: &V, raw: &Self::Raw,
    ) -> Result<Version, QueryError> {
        Ok(Version {
            main_class: raw.main_class.clone().into(),
            libraries: to_libraries(&raw.libraries),
            ..empty_version()
        })
    }
}

fn to_libraries(libs: &[MyLib]) -> Vec<Library> { todo!() }
fn empty_version() -> Version { todo!() }
```

The cache singleton (`MY_LOADER`) lets every call site reuse the same
repository — that's how Vanilla, Fabric, Forge, … all share their
respective caches at process level.

## Merging on top of Vanilla

Fabric / Quilt / NeoForge / Forge all call into the Vanilla repository
inside their `version_builder` and then layer their own libraries +
overrides on top. The pattern:

```rust,no_run
# use std::sync::Arc;
# use lighty_core::QueryError;
# use lighty_loaders::{VersionInfo, version_metadata::{Library, Version, VersionMetaData}};
# use lighty_loaders::utils::manifest::ManifestRepository;
# async fn doit<V: VersionInfo<LoaderType = lighty_loaders::Loader>>(version: &V) -> Result<Version, QueryError> {
#   use lighty_loaders::loaders::vanilla::vanilla::{VanillaQuery, VANILLA};
let vanilla: Arc<VersionMetaData> = VANILLA
    .get(version, VanillaQuery::VanillaBuilder)
    .await?;

let mut base = match vanilla.as_ref() {
    VersionMetaData::Version(v) => v.clone(),
    _ => return Err(QueryError::InvalidMetadata),
};

let my_libs: Vec<Library> = vec![/* … */];
base.libraries.extend(my_libs);
// override main_class if needed, merge JVM args, etc.

Ok(base)
# }
```

## Hooking into `LoaderExtensions`

Add a new `Loader::MyLoader` variant in `types::loader::loader`, then
add a dispatch arm in the auto-impl of `LoaderExtensions` so
`get_metadata` knows to call your repository.

## Event integration

Wrap the network call between `LoaderEvent::FetchingData` and
`LoaderEvent::DataFetched` so subscribers see your loader in the
event stream:

```rust,no_run
# use lighty_core::QueryError;
# use lighty_loaders::VersionInfo;
# #[cfg(feature = "events")]
# {
use lighty_event::{EVENT_BUS, Event, LoaderEvent};

# fn doit<V: VersionInfo>(version: &V) {
EVENT_BUS.emit(Event::Loader(LoaderEvent::FetchingData {
    loader: "MyLoader".into(),
    minecraft_version: version.minecraft_version().into(),
    loader_version:    version.loader_version().into(),
}));
# }
# }
```

See [`events.md`](./events.md) for the full variant list.

## See also

- [`traits.md`](./traits.md) — `VersionInfo` + `LoaderExtensions`
- [`cache.md`](./cache.md) — cache layer used by `ManifestRepository`
- [`events.md`](./events.md) — `LoaderEvent` variants
- Per-loader implementations:
  [`crates/loaders/src/loaders/*`](../src/loaders/) for live examples
