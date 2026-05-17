# Events

With the `events` feature on, modsloader emits its resolver, modpack
and post-install bucket events through the dedicated
`ModloaderEvent` enum (one module, one event family — `LaunchEvent` no
longer carries any `Mod*` / `Modpack*` variants). The resolver emits
**natively** from `resolver::resolve()`; the modpack pipeline and the
per-bucket install summaries are emitted from the launch crate. The
user-facing surface is a single `EventBus` plumbed through
`.launch(...).with_event_bus(&bus)`.

Feature wiring: the workspace `events` feature cascades through
`lighty-launch/events` → `lighty-modsloader/events` so consumer code
doesn't have to think about it.

## The full enum (`lighty_event::ModloaderEvent`)

Defined in `crates/event/src/module/modloader.rs`, wrapped at the root
by `Event::Modloader(ModloaderEvent)`.

### Mod resolver (native — `lighty_modsloader::resolver`)

| Variant | Field(s) | Emitted from |
|---|---|---|
| `ResolveStarted` | `request_count: usize` | Top of `resolve()`, before any HTTP call |
| `ResolveFetching` | `source: String, identifier: String` | Each unique `ModKey` visited in the BFS, just before its fetch |
| `ResolveDependency` | `parent: String, dependency: String` | Each `required` dep discovered while fetching `parent` |
| `ResolveCompleted` | `total_mods: usize` | End of `resolve()`, `total_mods` = `out.len()` |

### Modpack pipeline (launch — `crates/launch/src/installer/ressources/modpack`)

| Variant | Field(s) | Emitted from |
|---|---|---|
| `ModpackResolveStart` | `source: String` | Resolving the archive URL — human description of the `ModpackSource` variant |
| `ModpackArchiveDownloaded` | `sha1: String, bytes: u64` | After the `.mrpack` / `.zip` lands on disk (or is found in cache) |
| `ModpackOverridesExtracted` | `count: usize` | Reserved — currently emitted as part of `ModpackInstalled`'s log path; will become standalone in a future patch |
| `ModpackInstalled` | `name: String, mods_count: usize` | Pipeline complete; `mods_count` is the number of `Mods` entries queued for the standard installer |

### Post-install bucket summaries (launch — installer/ressources/mods.rs)

Emitted once the installer has finished writing files, broken down by
the sub-folder the asset routing dropped them into. Useful for UI
breakdowns ("12 mods, 3 resourcepacks, 1 shader installed").

| Variant | Field(s) | Meaning |
|---|---|---|
| `ResourcePacksInstalled` | `count: usize, bytes: u64` | Number + cumulative size of files under `<runtime>/resourcepacks/` |
| `ShaderPacksInstalled` | `count: usize, bytes: u64` | Same, for `<runtime>/shaderpacks/` |
| `DatapacksInstalled` | `count: usize, bytes: u64` | Same, for `<runtime>/datapacks/` |

(Plain `mods/` doesn't get its own bucket summary — `ModpackInstalled.mods_count` already covers that path.)

## Snippet

```rust
use lighty_event::{Event, EventBus, ModloaderEvent};
use lighty_launcher::prelude::*;

let bus = EventBus::new(1000);

tokio::spawn({
    let mut receiver = bus.subscribe();
    async move {
        while let Ok(event) = receiver.next().await {
            if let Event::Modloader(me) = event {
                match me {
                    ModloaderEvent::ResolveStarted { request_count } =>
                        println!("[Resolver] starting with {} requests", request_count),
                    ModloaderEvent::ModpackResolveStart { source } =>
                        println!("[Modpack] resolving {}", source),
                    ModloaderEvent::ModpackArchiveDownloaded { sha1, bytes } =>
                        println!("[Modpack] archive {} ({} bytes)", sha1, bytes),
                    ModloaderEvent::ModpackInstalled { name, mods_count } =>
                        println!("[Modpack] {} → {} mods queued", name, mods_count),
                    ModloaderEvent::ResourcePacksInstalled { count, bytes } =>
                        println!("[Install] {} resourcepacks ({} bytes)", count, bytes),
                    ModloaderEvent::ShaderPacksInstalled { count, bytes } =>
                        println!("[Install] {} shaders ({} bytes)", count, bytes),
                    ModloaderEvent::DatapacksInstalled { count, bytes } =>
                        println!("[Install] {} datapacks ({} bytes)", count, bytes),
                    _ => {}
                }
            }
        }
    }
});

// .launch(...).with_event_bus(&bus).run().await?;
```

## Migration note

Old code matching on `LaunchEvent::ModResolveStarted` /
`LaunchEvent::Modpack*` won't compile any more. Rename:

- `LaunchEvent::ModResolveStarted` → `ModloaderEvent::ResolveStarted`
- `LaunchEvent::ModResolveFetching` → `ModloaderEvent::ResolveFetching`
- `LaunchEvent::ModResolveDependency` → `ModloaderEvent::ResolveDependency`
- `LaunchEvent::ModResolveCompleted` → `ModloaderEvent::ResolveCompleted`
- `LaunchEvent::Modpack*` → `ModloaderEvent::Modpack*` (variant names unchanged)

and switch the outer match arm from `Event::Launch(...)` to
`Event::Modloader(...)`.
