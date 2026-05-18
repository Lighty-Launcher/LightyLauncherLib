# Events

`lighty-launch` emits two event families through the bus exposed by
`lighty-event`. Enable the `events` feature on `lighty-launch`
(and pass `.with_event_bus(&bus)` on the builder) to opt in.

The full cross-module catalogue lives in
[`crates/event/docs/events.md`](../../event/docs/events.md). This page
documents only the variants emitted by the launch pipeline.

| Family | Owner | What it covers |
|---|---|---|
| `LaunchEvent` | this crate (definition in `lighty-event`) | Install lifecycle + global byte progress + process spawn / stdio / exit |
| `ModloaderEvent` | mod resolution + modpack pipeline + per-bucket finishers (resource packs, shader packs, datapacks) |

Both are exported under `lighty_event::{LaunchEvent, ModloaderEvent}`
and re-exported as `lighty_launcher::event::*`.

## `LaunchEvent` variants

Defined in `crates/event/src/module/launch.rs`:

```rust,ignore
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

| Variant | When |
|---|---|
| `IsInstalled` | Every file already passed SHA1 — install short-circuits (natives are still re-extracted) |
| `InstallStarted` | First chunk about to be downloaded. `total_bytes` is the sum across all 8 buckets |
| `InstallProgress` | Per-chunk byte delta from the shared downloader. Sum client-side against `total_bytes` to drive a progress bar |
| `InstallCompleted` | All 8 buckets finished |
| `Launching` | About to spawn the JVM |
| `Launched` | Process spawned, carries the OS `pid` |
| `NotLaunched` | Spawn failed before the process started — `error` is human-readable |
| `ProcessOutput` | One line of stdout/stderr (`stream = "stdout" | "stderr"`) |
| `ProcessExited` | Process terminated; carries the final exit code |

## `ModloaderEvent` variants

Defined in `crates/event/src/module/modloader.rs`. Emitted by the
resolver, the modpack pipeline and the three mod-like buckets:

```rust,ignore
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

- `Resolve*` — fired during the BFS over Modrinth / CurseForge mod
  requests in `lighty_modsloader::resolver::resolve`.
- `Modpack*` — fired in order by the optional modpack pre-step:
  resolve → archive downloaded → overrides extracted → install
  complete.
- `{ResourcePacks,ShaderPacks,Datapacks}Installed` — per-bucket
  summaries fired once each bucket's parallel download finishes.
  `count` excludes files that were already valid on disk.

## Listening

```rust,no_run
# #[cfg(feature = "events")]
# {
use lighty_event::{Event, EventBus, LaunchEvent, ModloaderEvent};
use lighty_launch::launch::Launch;
# use lighty_auth::UserProfile;
# use lighty_core::AppState;
# use lighty_java::JavaDistribution;
# use lighty_launch::errors::InstallerResult;
# use lighty_loaders::types::Loader;
# use lighty_version::VersionBuilder;
# async fn run() -> InstallerResult<()> {
# AppState::init("MyLauncher").ok();
# let profile = UserProfile::offline("Player", "");
# let mut instance = VersionBuilder::new("inst", Loader::Vanilla, "", "1.21.1");

let bus = EventBus::new(1000);
let mut rx = bus.subscribe();

tokio::spawn(async move {
    while let Ok(event) = rx.next().await {
        match event {
            Event::Launch(LaunchEvent::InstallStarted { version, total_bytes }) =>
                println!("install {version} ({total_bytes} bytes)"),
            Event::Launch(LaunchEvent::InstallProgress { bytes }) =>
                println!("  +{bytes}"),
            Event::Launch(LaunchEvent::Launched { pid, .. }) =>
                println!("PID {pid}"),
            Event::Launch(LaunchEvent::ProcessOutput { pid, line, .. }) =>
                print!("[{pid}] {line}"),
            Event::Launch(LaunchEvent::ProcessExited { pid, exit_code }) =>
                println!("PID {pid} exited {exit_code}"),
            Event::Modloader(ModloaderEvent::ResolveCompleted { total_mods }) =>
                println!("resolved {total_mods} mods"),
            Event::Modloader(ModloaderEvent::ResourcePacksInstalled { count, bytes }) =>
                println!("resourcepacks: {count} / {bytes}"),
            _ => {}
        }
    }
});

instance.launch(&profile, JavaDistribution::Temurin)
    .with_event_bus(&bus)
    .run()
    .await?;
# Ok(()) }
# }
```

## Related

- [How to use](./how-to-use.md) — `with_event_bus` pattern
- [Installation](./installation.md) — where each event fires in the pipeline
- [Instance lifecycle](./instance-lifecycle.md) — `Launched` / `Exited` plumbing
- [Event catalogue](../../event/docs/events.md) — all modules
- [Auth events](../../auth/docs/events.md) — `AuthEvent` family
