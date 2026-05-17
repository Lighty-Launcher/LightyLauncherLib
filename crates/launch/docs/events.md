# Events

## Overview

`lighty-launch` emits two event families through the bus exposed by
`lighty-event`:

- **`LaunchEvent`** — install lifecycle + global byte progress +
  process spawning / exit. Owned by the launch crate.
- **`ModloaderEvent`** — dependency resolution, modpack pipeline and
  the per-bucket install summaries for resource packs, shader packs
  and datapacks. **The `ModResolve*` and `Modpack*` variants used to
  live under `LaunchEvent` — they have moved to `ModloaderEvent`.**

**Feature**: Requires the `events` feature flag.

**Exports**:
- `lighty_event::LaunchEvent`
- `lighty_event::ModloaderEvent`
- Re-exported under `lighty_launcher::event::{LaunchEvent, ModloaderEvent}`.

## LaunchEvent Variants

After the refacto, `LaunchEvent` only carries install/launch lifecycle
and process I/O. Every variant currently defined in
`crates/event/src/module/launch.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum LaunchEvent {
    IsInstalled       { version: String },
    InstallStarted    { version: String, total_bytes: u64 },
    InstallProgress   { bytes: u64 },
    InstallCompleted  { version: String, total_bytes: u64 },
    Launching         { version: String },
    Launched          { version: String, pid: u32 },
    NotLaunched       { version: String, error: String },
    ProcessOutput     { pid: u32, stream: String, line: String },
    ProcessExited     { pid: u32, exit_code: i32 },
}
```

### IsInstalled

Emitted when the verifier finds every file already valid on disk. The
installer skips the download phase entirely (only natives are
re-extracted, since they're cleaned on each run).

### InstallStarted / InstallProgress / InstallCompleted

Wrap the parallel download phase. `total_bytes` is the sum of every
missing/outdated file across the 8 buckets (libraries, client, assets,
mods, resourcepacks, shaderpacks, datapacks, natives). `InstallProgress`
is emitted by the shared downloader for each byte chunk written to disk —
sum the `bytes` field client-side to drive a progress bar.

### Launching / Launched / NotLaunched

Lifecycle around the Java process spawn. `Launched` carries the OS
`pid`; `NotLaunched` carries the error message when spawning failed
before the process started.

### ProcessOutput / ProcessExited

Per-line stdout/stderr stream (`stream = "stdout" | "stderr"`) and the
final exit code once the process terminates.

## ModloaderEvent Variants

Defined in `crates/event/src/module/modloader.rs`. Emitted by the
resolver, the modpack pipeline and the three new mod-like buckets:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum ModloaderEvent {
    ResolveStarted          { request_count: usize },
    ResolveFetching         { source: String, identifier: String },
    ResolveDependency       { parent: String, dependency: String },
    ResolveCompleted        { total_mods: usize },

    ModpackResolveStart       { source: String },
    ModpackArchiveDownloaded  { sha1: String, bytes: u64 },
    ModpackOverridesExtracted { count: usize },
    ModpackInstalled          { name: String, mods_count: usize },

    ResourcePacksInstalled { count: usize, bytes: u64 },
    ShaderPacksInstalled   { count: usize, bytes: u64 },
    DatapacksInstalled     { count: usize, bytes: u64 },
}
```

### Resolve* (dependency resolution)

Emitted by `lighty_modsloader::resolver::resolve` during the BFS over
Modrinth/CurseForge mod requests. `ResolveStarted` fires once with the
number of user-supplied requests; `ResolveFetching` fires per HTTP
fetch; `ResolveDependency` fires whenever a parent mod pulls a new
required dependency; `ResolveCompleted` fires once with the final mod
count after dedup.

### Modpack* (archive pipeline)

Emitted by the optional modpack stage (cf.
[`installation.md`](./installation.md)). Fires in order: resolve start
→ archive downloaded → overrides extracted → install complete.

### ResourcePacksInstalled / ShaderPacksInstalled / DatapacksInstalled

Per-bucket install summaries fired by the respective wrappers in
`crates/launch/src/installer/ressources/{resourcepacks,shaderpacks,datapacks}.rs`
once their `asset_partition::download` step finishes. `count` is the
number of files actually downloaded by this bucket (entries that
already passed SHA1 verification are excluded), `bytes` is the matching
byte total.

## Where the old variants went

| Removed from `LaunchEvent` | Now lives in |
|----------------------------|--------------|
| `ModResolveStarted`        | `ModloaderEvent::ResolveStarted` |
| `ModResolveFetching`       | `ModloaderEvent::ResolveFetching` |
| `ModResolveDependency`     | `ModloaderEvent::ResolveDependency` |
| `ModResolveCompleted`      | `ModloaderEvent::ResolveCompleted` |
| `ModpackResolveStart`      | `ModloaderEvent::ModpackResolveStart` |
| `ModpackArchiveDownloaded` | `ModloaderEvent::ModpackArchiveDownloaded` |
| `ModpackOverridesExtracted`| `ModloaderEvent::ModpackOverridesExtracted` |
| `ModpackInstalled`         | `ModloaderEvent::ModpackInstalled` |

Update callers that matched on `Event::Launch(LaunchEvent::ModResolve*)`
or `Event::Launch(LaunchEvent::Modpack*)` to match on
`Event::Modloader(ModloaderEvent::…)` instead.

## Complete Example

```rust
use lighty_event::{EventBus, Event, LaunchEvent, ModloaderEvent};
use lighty_launch::InstanceControl;
use lighty_core::AppState;
use lighty_launcher::prelude::*;
use lighty_java::JavaDistribution;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("MyLauncher")?;

    let event_bus = EventBus::new(1000);
    let mut receiver = event_bus.subscribe();

    tokio::spawn(async move {
        while let Ok(event) = receiver.next().await {
            match event {
                Event::Launch(LaunchEvent::InstallStarted { version, total_bytes }) => {
                    println!("Installing {} ({} bytes)", version, total_bytes);
                }
                Event::Launch(LaunchEvent::InstallProgress { bytes }) => {
                    println!("+{} bytes", bytes);
                }
                Event::Launch(LaunchEvent::InstallCompleted { version, .. }) => {
                    println!("Installed {}", version);
                }
                Event::Launch(LaunchEvent::Launched { version, pid }) => {
                    println!("Launched {} (PID: {})", version, pid);
                }
                Event::Launch(LaunchEvent::ProcessOutput { pid, stream, line }) => {
                    println!("[{} {}] {}", pid, stream, line);
                }
                Event::Launch(LaunchEvent::ProcessExited { pid, exit_code }) => {
                    println!("PID {} exited with code {}", pid, exit_code);
                }

                Event::Modloader(ModloaderEvent::ResolveCompleted { total_mods }) => {
                    println!("Resolved {} mods", total_mods);
                }
                Event::Modloader(ModloaderEvent::ResourcePacksInstalled { count, bytes }) => {
                    println!("ResourcePacks: {} files / {} bytes", count, bytes);
                }
                Event::Modloader(ModloaderEvent::ShaderPacksInstalled { count, bytes }) => {
                    println!("ShaderPacks: {} files / {} bytes", count, bytes);
                }
                Event::Modloader(ModloaderEvent::DatapacksInstalled { count, bytes }) => {
                    println!("Datapacks: {} files / {} bytes", count, bytes);
                }
                _ => {}
            }
        }
    });

    let mut instance = VersionBuilder::new(
        "fabric-1.21",
        Loader::Fabric,
        "0.16.9",
        "1.21.1",
    );

    let mut auth = OfflineAuth::new("Player");
    let profile = auth.authenticate().await?;

    instance.launch(&profile, JavaDistribution::Temurin)
        .with_event_bus(&event_bus)
        .run()
        .await?;

    Ok(())
}
```

## Related Documentation

- [How to Use](./how-to-use.md) - Practical examples with events
- [Exports](./exports.md) - Complete export reference
- [lighty-event Events](../../event/docs/events.md) - All event types
