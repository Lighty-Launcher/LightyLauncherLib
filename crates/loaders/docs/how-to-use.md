# Using lighty-loaders

The crate sits behind `VersionBuilder` / `LightyVersionBuilder` —
host code rarely calls it directly. The common path is:

1. Build a `VersionBuilder`.
2. Call `.get_metadata()` (from `LoaderExtensions`) to fetch + merge.
3. Hand the result to `lighty-launch`.

`VersionBuilder` itself lives in
[`lighty-version`](../../version/docs/how-to-use.md). What follows
focuses on the loader side.

## Minimum-viable fetch

```rust
use lighty_core::AppState;
use lighty_loaders::{Loader, LoaderExtensions};
use lighty_version::VersionBuilder;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("LightyLauncher")?;

    let instance = VersionBuilder::new(
        "vanilla-1.21",         // instance name
        Loader::Vanilla,
        "",                     // loader_version (Vanilla = empty)
        "1.21.1",               // minecraft_version
    );

    let metadata = instance.get_metadata().await?;
    println!("libraries: {}", metadata.libraries.len());
    Ok(())
}
```

`LoaderExtensions` also exposes:

- `get_libraries`
- `get_main_class`
- `get_natives`
- `get_java_version`
- `get_assets`

Each returns a partial `Arc<VersionMetaData>` populated only with the
requested slice. Detail in [`traits.md`](./traits.md).

## Loaders quick reference

```rust
use lighty_loaders::Loader;
use lighty_version::VersionBuilder;

VersionBuilder::new("vanilla",  Loader::Vanilla,  "",            "1.21.1");
VersionBuilder::new("fabric",   Loader::Fabric,   "0.16.9",      "1.21.1");
VersionBuilder::new("quilt",    Loader::Quilt,    "0.27.1",      "1.21.1");
VersionBuilder::new("neoforge", Loader::NeoForge, "21.8.53",     "1.21.8");
VersionBuilder::new("forge",    Loader::Forge,    "58.1.0",      "1.21.8");
VersionBuilder::new("forge12",  Loader::Forge,    "14.23.5.2860","1.12.2"); // legacy
```

For LightyUpdater (server-defined modpacks) use
`LightyVersionBuilder` — see [`loaders/lighty_updater.md`](./loaders/lighty_updater.md).

## Event subscriber

With the `events` feature on, every fetch / merge step emits a
`LoaderEvent`:

```rust
use lighty_event::{Event, EventBus, LoaderEvent};

let bus = EventBus::new(1000);
let mut rx = bus.subscribe();

tokio::spawn(async move {
    while let Ok(event) = rx.next().await {
        if let Event::Loader(le) = event {
            match le {
                LoaderEvent::FetchingData    { loader, .. } => println!("[fetch] {loader}"),
                LoaderEvent::DataFetched     { loader, .. } => println!("[fetched] {loader}"),
                LoaderEvent::ManifestCached  { loader }     => println!("[cache] {loader}"),
                LoaderEvent::MergingLoaderData { base_loader, overlay_loader } =>
                    println!("[merge] {overlay_loader} on top of {base_loader}"),
                _ => {}
            }
        }
    }
});
```

Full event catalogue in [`events.md`](./events.md).

## Instance size estimate

```rust
use lighty_loaders::{Loader, LoaderExtensions, InstanceSize};
use lighty_version::VersionBuilder;
use lighty_launcher::launch::InstanceControl;
# async fn run() -> anyhow::Result<()> {
let instance = VersionBuilder::new("v", Loader::Vanilla, "", "1.21.1");
let metadata = instance.get_metadata().await?;
let size = instance.size_of_instance(&metadata);
println!("total: {}", InstanceSize::format(size.total));
# Ok(()) }
```

`size_of_instance` lives on `InstanceControl` (the launch crate's
trait). `InstanceSize::format` formats a byte count as a human
string.

## Errors

```rust
use lighty_core::QueryError;
# fn handle(err: QueryError) {
match err {
    QueryError::Network(e)         => eprintln!("network: {e}"),
    QueryError::VersionNotFound { version } => eprintln!("no version {version}"),
    QueryError::UnsupportedLoader(l)        => eprintln!("loader {l} not enabled"),
    other                                   => eprintln!("{other}"),
}
# }
```

Full enum in
[`../../core/docs/exports.md`](../../core/docs/exports.md#errors).

## See also

- [`traits.md`](./traits.md) — `VersionInfo` + `LoaderExtensions`
- [`query.md`](./query.md) — extending the crate with a new loader
- [`cache.md`](./cache.md) — TTL + thundering-herd protection
- [`events.md`](./events.md) — `LoaderEvent` variants
- [`exports.md`](./exports.md) — public API
- Per-loader: [`loaders/`](./loaders/)
- [`../../version/docs/how-to-use.md`](../../version/docs/how-to-use.md)
  — building a `VersionBuilder`
- [`../../launch/docs/launch.md`](../../launch/docs/launch.md) —
  installing + running the resolved metadata
