# Event catalogue

Canonical list of every variant emitted in the LightyLauncher
workspace. Per-crate `events.md` files describe only their own
variants and link back here.

The root enum (`crates/event/src/lib.rs`) is:

```rust,ignore
pub enum Event {
    Auth(AuthEvent),
    Java(JavaEvent),
    Launch(LaunchEvent),
    Loader(LoaderEvent),
    Modloader(ModloaderEvent),
    Core(CoreEvent),
    InstanceLaunched(InstanceLaunchedEvent),
    InstanceWindowAppeared(InstanceWindowAppearedEvent),
    InstanceExited(InstanceExitedEvent),
    ConsoleOutput(ConsoleOutputEvent),
    InstanceDeleted(InstanceDeletedEvent),
}
```

## `AuthEvent` — `crates/auth`

| Variant | Fields |
|---|---|
| `AuthenticationStarted` | `provider: String` |
| `AuthenticationInProgress` | `provider: String, step: String` |
| `AuthenticationSuccess` | `provider: String, username: String, uuid: String` |
| `AuthenticationFailed` | `provider: String, error: String` |
| `AlreadyAuthenticated` | `provider: String, username: String` |

## `JavaEvent` — `crates/java`

| Variant | Fields |
|---|---|
| `JavaNotFound` | `distribution: String, version: u8` |
| `JavaAlreadyInstalled` | `distribution: String, version: u8, binary_path: String` |
| `JavaDownloadStarted` | `distribution: String, version: u8, total_bytes: u64` |
| `JavaDownloadProgress` | `bytes: u64` |
| `JavaDownloadCompleted` | `distribution: String, version: u8` |
| `JavaExtractionStarted` | `distribution: String, version: u8` |
| `JavaExtractionProgress` | `files_extracted: usize, total_files: usize` |
| `JavaExtractionCompleted` | `distribution: String, version: u8, binary_path: String` |

## `LaunchEvent` — `crates/launch`

Mod-source events used to live here. They moved to
[`ModloaderEvent`](#modloaderevent--cratesmodsloader--cratesinstaller).

| Variant | Fields |
|---|---|
| `IsInstalled` | `version: String` |
| `InstallStarted` | `version: String, total_bytes: u64` |
| `InstallProgress` | `bytes: u64` |
| `InstallCompleted` | `version: String, total_bytes: u64` |
| `Launching` | `version: String` |
| `Launched` | `version: String, pid: u32` |
| `NotLaunched` | `version: String, error: String` |
| `ProcessOutput` | `pid: u32, stream: String, line: String` |
| `ProcessExited` | `pid: u32, exit_code: i32` |

## `LoaderEvent` — `crates/loaders`

| Variant | Fields |
|---|---|
| `FetchingData` | `loader: String, minecraft_version: String, loader_version: String` |
| `DataFetched` | `loader: String, minecraft_version: String, loader_version: String` |
| `ManifestNotFound` | `loader, minecraft_version, loader_version, error: String` |
| `ManifestCached` | `loader: String` |
| `MergingLoaderData` | `base_loader: String, overlay_loader: String` |
| `DataMerged` | `base_loader: String, overlay_loader: String` |

## `ModloaderEvent` — `crates/modsloader` + `crates/launch/installer`

Split from `LaunchEvent` when the mod-source pipeline became its own
module. The resolver variants are emitted natively from
`lighty_modsloader::resolver`; the modpack pipeline and per-bucket
summaries are emitted from the launch installer.

| Variant | Fields |
|---|---|
| `ResolveStarted` | `request_count: usize` |
| `ResolveFetching` | `source: String, identifier: String` |
| `ResolveDependency` | `parent: String, dependency: String` |
| `ResolveCompleted` | `total_mods: usize` |
| `ModpackResolveStart` | `source: String` |
| `ModpackArchiveDownloaded` | `sha1: String, bytes: u64` |
| `ModpackOverridesExtracted` | `count: usize` |
| `ModpackInstalled` | `name: String, mods_count: usize` |
| `ResourcePacksInstalled` | `count: usize, bytes: u64` |
| `ShaderPacksInstalled` | `count: usize, bytes: u64` |
| `DatapacksInstalled` | `count: usize, bytes: u64` |

## `CoreEvent` — `crates/core`

| Variant | Fields |
|---|---|
| `ExtractionStarted` | `archive_type: String, file_count: usize, destination: String` |
| `ExtractionProgress` | `files_extracted: usize, total_files: usize` |
| `ExtractionCompleted` | `archive_type: String, files_extracted: usize` |

## Instance + console events

Defined in `crates/event/src/module/console.rs`. Each carries a
`SystemTime` timestamp (serialized as Unix seconds).

| Root variant | Struct fields |
|---|---|
| `InstanceLaunched(InstanceLaunchedEvent)` | `pid, instance_name, version, username, timestamp` |
| `InstanceWindowAppeared(InstanceWindowAppearedEvent)` | `pid, instance_name, version, timestamp` |
| `InstanceExited(InstanceExitedEvent)` | `pid, instance_name, exit_code: Option<i32>, timestamp` |
| `ConsoleOutput(ConsoleOutputEvent)` | `pid, instance_name, stream: ConsoleStream, line, timestamp` |
| `InstanceDeleted(InstanceDeletedEvent)` | `instance_name, timestamp` |

`ConsoleStream` is `Stdout | Stderr` (lowercase in JSON).

## Migration notes

- `LaunchEvent::ModResolveStarted/Fetching/Dependency/Completed` →
  `ModloaderEvent::Resolve*`
- `LaunchEvent::Modpack*` → `ModloaderEvent::Modpack*` (variant names
  unchanged, outer arm changed from `Event::Launch(_)` to
  `Event::Modloader(_)`)

## See also

- [`architecture.md`](./architecture.md) — event bus + receivers
- [`modules.md`](./modules.md) — file layout under `module/`
- [`how-to-use.md`](./how-to-use.md) — subscribe / filter / fan-out patterns
