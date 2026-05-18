# Events

With the `events` feature on, `lighty-modsloader` emits resolver,
modpack and per-bucket install events through `ModloaderEvent`. The
resolver emits **natively** from `resolver::resolve()`; the modpack
pipeline and per-bucket summaries are emitted from the launch crate.
The user-facing surface is one `EventBus` plumbed through
`.launch(...).with_event_bus(&bus)`.

Feature wiring: the workspace `events` feature cascades through
`lighty-launch/events` → `lighty-modsloader/events`, so consumer
code doesn't have to think about it.

The full workspace event catalogue is in
[`../../event/docs/events.md`](../../event/docs/events.md). The
variants this crate triggers:

## Mod resolver (native — `lighty_modsloader::resolver`)

| Variant | Fields | Where |
|---|---|---|
| `ResolveStarted` | `request_count: usize` | Top of `resolve()`, before any HTTP call |
| `ResolveFetching` | `source: String, identifier: String` | Each unique `ModKey` visited in the BFS, just before its fetch |
| `ResolveDependency` | `parent: String, dependency: String` | Each `required` dep discovered while fetching `parent` |
| `ResolveCompleted` | `total_mods: usize` | End of `resolve()`; `total_mods` = output length |

## Modpack pipeline (launch — `crates/launch/src/installer/ressources/modpack`)

| Variant | Fields | Where |
|---|---|---|
| `ModpackResolveStart` | `source: String` | URL resolution — human description of the `ModpackSource` variant |
| `ModpackArchiveDownloaded` | `sha1: String, bytes: u64` | After the `.mrpack` / `.zip` lands on disk (or hits cache) |
| `ModpackOverridesExtracted` | `count: usize` | Reserved — currently folded into `ModpackInstalled` |
| `ModpackInstalled` | `name: String, mods_count: usize` | Pipeline complete; `mods_count` is the number of `Mods` entries queued for the standard installer |

## Per-bucket install summaries (launch — `installer/ressources/mods.rs`)

Emitted once the installer has finished writing files, broken down by
the sub-folder asset routing put them into. Useful for UI breakdowns
("12 mods, 3 resourcepacks, 1 shader installed").

| Variant | Fields | Meaning |
|---|---|---|
| `ResourcePacksInstalled` | `count, bytes` | Files under `<runtime>/resourcepacks/` |
| `ShaderPacksInstalled` | `count, bytes` | Files under `<runtime>/shaderpacks/` |
| `DatapacksInstalled` | `count, bytes` | Files under `<runtime>/datapacks/` |

Plain `mods/` doesn't get its own bucket summary — `ModpackInstalled.
mods_count` already covers that.

## Snippet

```rust
use lighty_event::{Event, EventBus, ModloaderEvent};

let bus = EventBus::new(1000);
let mut rx = bus.subscribe();

tokio::spawn(async move {
    while let Ok(event) = rx.next().await {
        if let Event::Modloader(me) = event {
            match me {
                ModloaderEvent::ResolveStarted    { request_count } =>
                    println!("[resolve] {request_count} requests"),
                ModloaderEvent::ModpackResolveStart { source } =>
                    println!("[pack]    resolving {source}"),
                ModloaderEvent::ModpackInstalled  { name, mods_count } =>
                    println!("[pack]    {name} -> {mods_count} mods queued"),
                ModloaderEvent::ResourcePacksInstalled { count, bytes } =>
                    println!("[install] {count} resourcepacks ({bytes} B)"),
                _ => {}
            }
        }
    }
});

// .launch(...).with_event_bus(&bus).run().await?
```

## Migration note

Old code matching on `LaunchEvent::ModResolveStarted` /
`LaunchEvent::Modpack*` won't compile. Rename:

- `LaunchEvent::ModResolveStarted` → `ModloaderEvent::ResolveStarted`
- `LaunchEvent::ModResolveFetching` → `ModloaderEvent::ResolveFetching`
- `LaunchEvent::ModResolveDependency` → `ModloaderEvent::ResolveDependency`
- `LaunchEvent::ModResolveCompleted` → `ModloaderEvent::ResolveCompleted`
- `LaunchEvent::Modpack*` → `ModloaderEvent::Modpack*` (variant names unchanged)

And switch the outer match arm from `Event::Launch(_)` to
`Event::Modloader(_)`.

## See also

- [`../../event/docs/events.md`](../../event/docs/events.md) — full workspace catalogue
- [`mods.md`](./mods.md) — resolver behaviour
- [`modpacks.md`](./modpacks.md) — modpack pipeline detail
