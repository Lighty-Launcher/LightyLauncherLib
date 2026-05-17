# Event Reference

Every module event ultimately surfaces through the root `Event` enum
(`crates/event/src/lib.rs`):

```rust
pub enum Event {
    Auth(AuthEvent),
    Java(JavaEvent),
    Launch(LaunchEvent),
    Loader(LoaderEvent),
    Modloader(ModloaderEvent),   // new — split out of LaunchEvent
    Core(CoreEvent),
    InstanceLaunched(InstanceLaunchedEvent),
    InstanceWindowAppeared(InstanceWindowAppearedEvent),
    InstanceExited(InstanceExitedEvent),
    ConsoleOutput(ConsoleOutputEvent),
    InstanceDeleted(InstanceDeletedEvent),
}
```

## Event Categories

### AuthEvent
- `AuthenticationStarted` - Login begins
- `AuthenticationSuccess` - Login successful
- `AuthenticationFailed` - Login failed

### JavaEvent
- `JavaDownloadStarted` - JRE download begins
- `JavaDownloadProgress` - Download progress
- `JavaDownloadCompleted` - Download complete
- `JavaExtractionStarted` - Extraction begins
- `JavaExtractionCompleted` - Extraction complete

### LaunchEvent
- `IsInstalled` - All install targets already up-to-date
- `InstallStarted` - Installation begins
- `InstallProgress` - Byte counter increment (global)
- `InstallCompleted` - Installation complete
- `Launching` - About to spawn the JVM
- `Launched` - JVM spawned (carries `pid`)
- `NotLaunched` - Spawning the JVM failed
- `ProcessOutput` - stdout/stderr line (carries `pid`, `stream`)
- `ProcessExited` - Process terminated (carries `pid`, `exit_code`)

### LoaderEvent
- `FetchingData` - Fetching loader manifest
- `DataFetched` - Manifest retrieved
- `ManifestCached` - Using cached data

### ModloaderEvent
New module dedicated to mod-source side effects. Defined in
`crates/event/src/module/modloader.rs`.

- `ResolveStarted` - BFS started (`request_count`)
- `ResolveFetching` - Per-fetch trace (`source`, `identifier`)
- `ResolveDependency` - Parent pulled a dependency
- `ResolveCompleted` - BFS done (`total_mods`)
- `ModpackResolveStart` - Modpack archive URL resolution started
- `ModpackArchiveDownloaded` - Archive cached on disk (`sha1`, `bytes`)
- `ModpackOverridesExtracted` - Number of overrides copied
- `ModpackInstalled` - Modpack pipeline complete (`name`, `mods_count`)
- `ResourcePacksInstalled` - Resource-pack bucket summary (`count`, `bytes`)
- `ShaderPacksInstalled` - Shader-pack bucket summary (`count`, `bytes`)
- `DatapacksInstalled` - Datapack bucket summary (`count`, `bytes`)

The `ResolveStarted/Fetching/Dependency/Completed` and `Modpack*`
variants used to live under `LaunchEvent`; they were moved here when
the mod-source pipeline got its own module.

### CoreEvent
- `DownloadStarted` - File download begins
- `DownloadProgress` - Download progress
- `ExtractionStarted` - Archive extraction begins

### InstanceEvent
- `InstanceLaunched` - Instance started (PID, version, username)
- `ConsoleOutput` - Real-time stdout/stderr
- `InstanceExited` - Instance exited (exit code)
- `InstanceDeleted` - Instance deleted

## See Also

- [Architecture](./architecture.md)
- [Examples](./examples.md)
