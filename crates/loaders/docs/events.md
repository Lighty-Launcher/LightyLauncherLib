# LoaderEvent

Six events emitted by `lighty-loaders` when the `events` feature is
on. They surface through the global bus as
`Event::Loader(LoaderEvent::…)`.

Defined in `crates/event/src/module/loader.rs`. Full workspace
catalogue:
[`../../event/docs/events.md`](../../event/docs/events.md).

## Variants

| Variant | Fields | When |
|---|---|---|
| `FetchingData` | `loader, minecraft_version, loader_version` | Before HTTP fetch of a loader manifest |
| `DataFetched` | `loader, minecraft_version, loader_version` | After the manifest lands and parses |
| `ManifestNotFound` | `loader, minecraft_version, loader_version, error` | Provider returned 404 or version missing |
| `ManifestCached` | `loader` | Repository hit the cache instead of fetching |
| `MergingLoaderData` | `base_loader, overlay_loader` | Before a Fabric / Quilt / Forge / NeoForge merge with Vanilla |
| `DataMerged` | `base_loader, overlay_loader` | After the merge completes |

`loader` is the human name: `"Vanilla"`, `"Fabric"`, `"Quilt"`,
`"NeoForge"`, `"Forge"`, `"LightyUpdater"`.

## Typical sequences

**Cold fetch** (first time, no cache):
```
FetchingData { loader: "Fabric", ... }
DataFetched   { loader: "Fabric", ... }
MergingLoaderData { base_loader: "Vanilla", overlay_loader: "Fabric" }
DataMerged   { base_loader: "Vanilla", overlay_loader: "Fabric" }
```

**Warm** (cache hit):
```
ManifestCached { loader: "Fabric" }
```

**Partial** (Vanilla cached, Fabric not):
```
ManifestCached { loader: "Vanilla" }
FetchingData   { loader: "Fabric", ... }
DataFetched    { loader: "Fabric", ... }
MergingLoaderData ...
DataMerged ...
```

**Not found**:
```
FetchingData      { loader: "Fabric", loader_version: "0.99", ... }
ManifestNotFound  { loader: "Fabric", error: "...", ... }
```

## Subscriber

```rust
use lighty_event::{Event, EventBus, LoaderEvent};

let bus = EventBus::new(1000);
let mut rx = bus.subscribe();

tokio::spawn(async move {
    while let Ok(event) = rx.next().await {
        if let Event::Loader(le) = event {
            match le {
                LoaderEvent::FetchingData     { loader, .. } => println!("[fetch] {loader}"),
                LoaderEvent::DataFetched      { loader, .. } => println!("[ok]    {loader}"),
                LoaderEvent::ManifestNotFound { loader, error, .. } => eprintln!("[404]  {loader}: {error}"),
                LoaderEvent::ManifestCached   { loader }     => println!("[cache] {loader}"),
                LoaderEvent::MergingLoaderData { base_loader, overlay_loader } =>
                    println!("[merge] {overlay_loader} <- {base_loader}"),
                LoaderEvent::DataMerged       { base_loader, overlay_loader } =>
                    println!("[merged] {overlay_loader} <- {base_loader}"),
            }
        }
    }
});
```

## See also

- [`cache.md`](./cache.md) — what `ManifestCached` actually means
- [`query.md`](./query.md) — where the fetch / merge events fire from
- [`../../event/docs/events.md`](../../event/docs/events.md) — full workspace catalogue
