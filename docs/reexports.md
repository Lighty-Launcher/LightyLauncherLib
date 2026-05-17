# Re-exports Reference

Complete reference of all types and functions re-exported by `lighty-launcher`.

## Module Organization

```rust
lighty-launcher
├── auth          // Authentication providers
├── event         // Event system (with "events" feature)
├── java          // Java runtime management
├── launch        // Game launching and installation
├── loaders       // Mod loader implementations
├── mods          // Mods + modpacks (Modrinth / CurseForge / .mrpack / .zip)
├── version       // Version builders
├── core          // Core utilities
├── macros        // Utility macros
└── prelude       // Common imports
```

## Authentication Module (`lighty_launcher::auth`)

**Source**: [`lighty-auth`](../crates/auth/README.md)

### Types

| Type | Description | Defined In |
|------|-------------|------------|
| `Authenticator` | Trait for authentication providers | `lighty_auth` |
| `UserProfile` | User authentication data | `lighty_auth` |
| `UserRole` | User role (User, Admin) | `lighty_auth` |
| `AuthProvider` | Authentication provider enum | `lighty_auth` |
| `AuthResult<T>` | Result type for authentication operations | `lighty_auth` |
| `AuthError` | Authentication error types | `lighty_auth` |

### Providers

| Provider | Description |
|----------|-------------|
| `OfflineAuth` | Offline authentication with UUID v5 generation |
| `MicrosoftAuth` | Microsoft OAuth 2.0 authentication |
| `AzuriomAuth` | Azuriom CMS integration |

### Functions

| Function | Description |
|----------|-------------|
| `generate_offline_uuid(username: &str)` | Generate deterministic UUID for offline mode |

**Usage**:
```rust
use lighty_launcher::auth::{OfflineAuth, Authenticator};

let mut auth = OfflineAuth::new("Player");
let profile = auth.authenticate().await?;
```

**Detailed docs**: [lighty-auth](../crates/auth/README.md)

---

## Events Module (`lighty_launcher::event`)

**Source**: [`lighty-event`](../crates/event/README.md)
**Requires**: `events` feature

### Core Types

| Type | Description |
|------|-------------|
| `EventBus` | Multi-producer, multi-consumer event bus |
| `EventReceiver` | Receiver for events (from `subscribe()`) |
| `Event` | Main event enum |

### Event Types

| Event | Description |
|-------|-------------|
| `AuthEvent` | Authentication events |
| `JavaEvent` | Java download/installation events |
| `LaunchEvent` | Game installation/launch events (no longer carries `Mod*` / `Modpack*` variants — those moved to `ModloaderEvent`) |
| `LoaderEvent` | Loader metadata fetching events |
| `ModloaderEvent` | Mod resolver + modpack pipeline + per-bucket install summaries (resourcepacks / shaderpacks / datapacks). Wrapped as `Event::Modloader(...)`. Not yet re-exported by `lighty_launcher::event` — import via `lighty_event::ModloaderEvent` for now. |
| `CoreEvent` | Core operations (extraction, etc.) |

### Instance Events

| Event | Description |
|-------|-------------|
| `InstanceLaunchedEvent` | Game instance launched |
| `InstanceExitedEvent` | Game instance exited |
| `ConsoleOutputEvent` | Console output line |
| `InstanceDeletedEvent` | Instance deleted |

### Other Types

| Type | Description |
|------|-------------|
| `ConsoleStream` | Stdout or Stderr |
| `EventReceiveError` | Error receiving events |
| `EventTryReceiveError` | Error trying to receive |
| `EventSendError` | Error sending events |
| `EVENT_BUS` | Global event bus singleton |

**Usage**:
```rust
use lighty_launcher::event::{EventBus, Event};

let event_bus = EventBus::new(1000);
let mut receiver = event_bus.subscribe();

tokio::spawn(async move {
    while let Ok(event) = receiver.next().await {
        println!("Event: {:?}", event);
    }
});
```

**Detailed docs**: [lighty-event](../crates/event/README.md)

---

## Java Module (`lighty_launcher::java`)

**Source**: [`lighty-java`](../crates/java/README.md)

### Types

| Type | Description |
|------|-------------|
| `JavaDistribution` | Java distribution enum (Temurin, GraalVM, Zulu, Liberica) |
| `DistributionSelection` | Distribution selection helper |
| `JavaRuntime` | Java process executor |
| `JreError` | JRE download errors |
| `JreResult<T>` | Result type for JRE operations |
| `JavaRuntimeError` | Runtime execution errors |
| `JavaRuntimeResult<T>` | Result type for runtime operations |
| `DistributionError` | Distribution errors |
| `DistributionResult<T>` | Result type for distribution operations |

### Modules

| Module | Description |
|--------|-------------|
| `jre_downloader` | JRE download and installation functions |

**Usage**:
```rust
use lighty_launcher::java::JavaDistribution;

let distribution = JavaDistribution::Temurin;
// Automatically downloaded and managed by launch system
```

**Detailed docs**: [lighty-java](../crates/java/README.md)

---

## Launch Module (`lighty_launcher::launch`)

**Source**: [`lighty-launch`](../crates/launch/README.md)

### Traits

| Trait | Description |
|-------|-------------|
| `Launch` | Launch builder trait (auto-implemented for VersionInfo types) |
| `Installer` | Installation trait |
| `InstanceControl` | Instance management trait (⚠️ must import!) |

### Types

| Type | Description |
|------|-------------|
| `LaunchBuilder` | Fluent API for configuring launch |
| `LaunchConfig` | Launch configuration |
| `DownloaderConfig` | Download configuration (max retries, concurrency) |
| `LaunchArguments` | Argument building trait |

### Functions

| Function | Description |
|----------|-------------|
| `init_downloader_config(config: DownloaderConfig)` | Configure global downloader settings |

### Errors

| Type | Description |
|------|-------------|
| `InstallerError` | Installation errors |
| `InstallerResult<T>` | Result type for installer operations |
| `InstanceError` | Instance management errors |
| `InstanceResult<T>` | Result type for instance operations |

### Launch Keys (`lighty_launcher::launch::keys`)

All placeholder constants for argument customization:

| Constant | Placeholder | Value |
|----------|-------------|-------|
| `KEY_AUTH_PLAYER_NAME` | `${auth_player_name}` | `"auth_player_name"` |
| `KEY_AUTH_UUID` | `${auth_uuid}` | `"auth_uuid"` |
| `KEY_AUTH_ACCESS_TOKEN` | `${auth_access_token}` | `"auth_access_token"` |
| `KEY_AUTH_XUID` | `${auth_xuid}` | `"auth_xuid"` |
| `KEY_CLIENT_ID` | `${clientid}` | `"clientid"` |
| `KEY_USER_TYPE` | `${user_type}` | `"user_type"` |
| `KEY_USER_PROPERTIES` | `${user_properties}` | `"user_properties"` |
| `KEY_VERSION_NAME` | `${version_name}` | `"version_name"` |
| `KEY_VERSION_TYPE` | `${version_type}` | `"version_type"` |
| `KEY_GAME_DIRECTORY` | `${game_directory}` | `"game_directory"` |
| `KEY_ASSETS_ROOT` | `${assets_root}` | `"assets_root"` |
| `KEY_NATIVES_DIRECTORY` | `${natives_directory}` | `"natives_directory"` |
| `KEY_LIBRARY_DIRECTORY` | `${library_directory}` | `"library_directory"` |
| `KEY_ASSETS_INDEX_NAME` | `${assets_index_name}` | `"assets_index_name"` |
| `KEY_LAUNCHER_NAME` | `${launcher_name}` | `"launcher_name"` |
| `KEY_LAUNCHER_VERSION` | `${launcher_version}` | `"launcher_version"` |
| `KEY_CLASSPATH` | `${classpath}` | `"classpath"` |
| `KEY_CLASSPATH_SEPARATOR` | `${classpath_separator}` | `"classpath_separator"` |

**Usage**:
```rust
use lighty_launcher::launch::{Launch, InstanceControl};
use lighty_launcher::launch::keys::*;

// Launch
instance.launch(&profile, JavaDistribution::Temurin)
    .with_arguments()
        .set(KEY_LAUNCHER_NAME, "MyLauncher")
        .done()
    .run()
    .await?;

// Instance control (must import trait!)
if let Some(pid) = instance.get_pid() {
    instance.close_instance(pid).await?;
}
```

**Detailed docs**: [lighty-launch](../crates/launch/README.md)

---

## Loaders Module (`lighty_launcher::loaders`)

**Source**: [`lighty-loaders`](../crates/loaders/README.md)

### Types

| Type | Description |
|------|-------------|
| `Loader` | Loader enum (Vanilla, Fabric, Quilt, Forge, NeoForge, etc.) |
| `VersionInfo` | Trait for version information |
| `LoaderExtensions` | Trait for loader operations (auto-implemented) |
| `InstanceSize` | Instance size calculation |

### Version Metadata

| Type | Description |
|------|-------------|
| `Version` | Complete version metadata |
| `VersionMetaData` | Version metadata wrapper |
| `Library` | Library dependency |
| `Asset` | Asset file |
| `AssetIndex` | Asset index |
| `Arguments` | Game/JVM arguments |
| `MainClass` | Main class info |
| `Mods` | Mod list |
| `Native` | Native library |

### Loader Implementations

| Module | Loader |
|--------|--------|
| `vanilla` | Vanilla Minecraft |
| `fabric` | Fabric mod loader |
| `quilt` | Quilt mod loader |
| `forge` | Forge mod loader |
| `neoforge` | NeoForge mod loader |
| `lighty_updater` | LightyUpdater custom server |
| `optifine` | OptiFine |

### Utilities

| Module | Description |
|--------|-------------|
| `cache` | Manifest caching utilities |
| `manifest` | Manifest fetching |
| `query` | Version querying |

(The unified `QueryError` lives in `lighty_core` — no longer under
`lighty_loaders::utils::error`.)

**Usage**:
```rust
use lighty_launcher::loaders::{Loader, VersionInfo, LoaderExtensions};

let loader = Loader::Fabric;

// VersionInfo trait provides metadata access
let name = instance.name();
let mc_version = instance.minecraft_version();

// LoaderExtensions provides async operations (auto-implemented)
let metadata = instance.get_metadata().await?;
let libraries = instance.get_libraries().await?;
```

**Detailed docs**: [lighty-loaders](../crates/loaders/README.md)

---

## Mods Module (`lighty_launcher::mods`)

**Source**: [`lighty-modsloader`](../crates/modsloader/docs/overview.md)

Exposes the mods + modpacks subsystem. Pulled in automatically as soon
as `modrinth` or `curseforge` is enabled — each provider feature also
carries its modpack format parser (no separate `modpack` feature).

### Always available

| Type | Description |
|------|-------------|
| `ModRequest` | User-facing request (`Modrinth { id_or_slug, version }` or `CurseForge { mod_id, file_id }`) |
| `ModSource` | Tag enum: `Modrinth` / `CurseForge` |
| `ModKey` | Dedup key — `(source, project id)`, version-agnostic |
| `WithMods` | Trait the launch pipeline bounds against (`VersionBuilder` implements it) |

### With `modrinth` or `curseforge`

| Type | Description |
|------|-------------|
| `ModpackSource` | `ModrinthUrl(url)` / `ModrinthPinned { project, version }` / `CurseForgePinned { project_id, file_id }` |

### Submodules (feature-gated)

| Module | Feature | Description |
|--------|---------|-------------|
| `modrinth` | `modrinth` | Labrinth-API client (`fetch`), wire types, `.mrpack` parser. Sub-modules: `api`, `client`, `client_metadata`, `modpack`, `modpack_metadata` |
| `curseforge` | `curseforge` | Core-API client (`fetch`, `set_api_key`, `fetch_pinned_file`, `install_subdir_for`), constants, `.zip` parser. Sub-modules: `api`, `client`, `client_metadata`, `modpack`, `modpack_metadata` |
| `resolver` | `modrinth` or `curseforge` | BFS dependency resolver (native `ModloaderEvent::Resolve*` emission with `events`) |
| `modpack` | `modrinth` or `curseforge` | Flat module file holding the `ModpackSource` enum + `From<&str>` / `From<String>` impls — no whitelist helper, no nested provider modules (those moved next to their clients) |

**Asset routing**: clients route each fetched asset to its sub-folder
(`mods/`, `resourcepacks/`, `shaderpacks/`, `datapacks/`) by reading
the Modrinth `project_type` / CurseForge `classId`. Unknown types
surface `QueryError::UnsupportedFormat`. See
[`crates/modsloader/docs/mods.md`](../crates/modsloader/docs/mods.md)
and [`ASSETS_ROUTING.md`](../ASSETS_ROUTING.md).

**Usage**:

```rust
use lighty_launcher::mods::{ModRequest, ModpackSource};

let instance = VersionBuilder::new("mc", Loader::Fabric, "0.16.9", "1.21.1")
    .with_mod()
        .with_modrinth_mods(vec![("sodium", None)])
        .with_modrinth_modpack("https://cdn.modrinth.com/.../pack.mrpack")
        .done();
```

> Back-compat: `lighty_launcher::loaders::mods` still resolves to the
> standalone `lighty_modsloader` crate. New code should prefer the
> top-level `lighty_launcher::mods` re-export.

**Detailed docs**: [lighty-modsloader/docs](../crates/modsloader/docs/overview.md)

---

## Version Module (`lighty_launcher::version`)

**Source**: [`lighty-version`](../crates/version/README.md)

### Types

| Type | Description |
|------|-------------|
| `VersionBuilder` | Standard version builder for all loaders |
| `LightyVersionBuilder` | Custom server version builder |

**Usage**:
```rust
use lighty_launcher::version::VersionBuilder;

let instance = VersionBuilder::new(
    "my-instance",
    Loader::Vanilla,
    "",
    "1.21.1",
);
```

**Detailed docs**: [lighty-version](../crates/version/README.md)

---

## Core Module (`lighty_launcher::core`)

**Source**: [`lighty-core`](../crates/core/README.md)

### Types

| Type | Description |
|------|-------------|
| `AppState` | Application state and project directories |
| `AppStateError` | AppState errors |
| `AppStateResult<T>` | AppState result type |
| `SystemError` | System operation errors |
| `SystemResult<T>` | System result type |
| `ExtractError` | Archive extraction errors |
| `ExtractResult<T>` | Extract result type |
| `DownloadError` | Download errors |
| `DownloadResult<T>` | Download result type |
| `HashError` | Hashing errors |
| `HashResult<T>` | Hash result type |

### Modules

| Module | Description |
|--------|-------------|
| `system` | System detection and operations |
| `hosts` | Hosts file management |
| `download` | HTTP download utilities |
| `extract` | Archive extraction (ZIP, TAR.GZ) |
| `hash` | SHA1 hashing utilities |

### Functions

| Function | Description |
|----------|-------------|
| `verify_file_sha1(path, hash)` | Verify file SHA1 (async) |
| `verify_file_sha1_streaming(path, hash)` | Verify with streaming |
| `calculate_file_sha1_sync(path)` | Calculate SHA1 (blocking) |
| `verify_file_sha1_sync(path, hash)` | Verify SHA1 (blocking) |
| `calculate_sha1_bytes(bytes)` | Calculate SHA1 from bytes |
| `calculate_sha1_bytes_raw(bytes)` | Raw SHA1 calculation |

**Usage**:
```rust
use lighty_launcher::core::AppState;

AppState::init("MyLauncher")?;

let data_dir = AppState::data_dir();
let cache_dir = AppState::cache_dir();
```

**Detailed docs**: [lighty-core](../crates/core/README.md)

---

## Macros Module (`lighty_launcher::macros`)

**Source**: `lighty-core`

### Tracing Macros

| Macro | Description | Requires Feature |
|-------|-------------|------------------|
| `trace_debug!()` | Debug level logging | `tracing` (no-op otherwise) |
| `trace_info!()` | Info level logging | `tracing` (no-op otherwise) |
| `trace_warn!()` | Warning level logging | `tracing` (no-op otherwise) |
| `trace_error!()` | Error level logging | `tracing` (no-op otherwise) |
| `time_it!(name, block)` | Performance timing | `tracing` (no-op otherwise) |

### File System Macros

| Macro | Description |
|-------|-------------|
| `mkdir!(path)` | Async directory creation with error logging |
| `join_and_mkdir!(base, segment)` | Join path and create directory |
| `join_and_mkdir_vec!(base, segments)` | Join multiple segments and create |
| `mkdir_blocking!(path)` | Blocking directory creation |

**Usage**:
```rust
use lighty_launcher::macros::*;

trace_info!("Starting operation");

let result = time_it!("my_operation", {
    // Expensive operation
    42
});

let path = std::path::Path::new("/tmp/my_dir");
mkdir!(path);
```

---

## Prelude (`lighty_launcher::prelude`)

Convenient re-exports of most commonly used types.

### Included Types

```rust
use lighty_launcher::prelude::*;

// Authentication
Authenticator, UserProfile, AuthProvider, AuthError, UserRole,
OfflineAuth, MicrosoftAuth, AzuriomAuth

// Events (with "events" feature)
EventBus, Event, AuthEvent, JavaEvent, LaunchEvent, LoaderEvent, CoreEvent,
InstanceLaunchedEvent, InstanceExitedEvent, ConsoleOutputEvent,
InstanceDeletedEvent, ConsoleStream, EVENT_BUS
// ModloaderEvent is NOT in the prelude (nor in `lighty_launcher::event`
// today). Reach it directly via `use lighty_event::ModloaderEvent;` —
// adding it to the facade is a pending follow-up.

// Java
JavaDistribution

// Launch
Launch, LaunchBuilder, DownloaderConfig, init_downloader_config,
InstanceControl, InstanceError, InstanceResult

// Launch keys (all KEY_* constants)

// Loaders
Loader, VersionInfo, LoaderExtensions, InstanceSize

// Version
VersionBuilder, LightyVersionBuilder

// Core
AppState

// Macros
trace_debug, trace_info, trace_warn, trace_error
```

**Usage**:
```rust
use lighty_launcher::prelude::*;

// All common types are now available
let mut auth = OfflineAuth::new("Player");
let instance = VersionBuilder::new(...);
```

---

## Root Re-exports

Most commonly used types are also available at the root:

```rust
use lighty_launcher::{
    Loader,               // loaders::Loader
    JavaDistribution,     // java::JavaDistribution
    Launch,               // launch::Launch
    Authenticator,        // auth::Authenticator
    UserProfile,          // auth::UserProfile
    VersionBuilder,       // version::VersionBuilder
    LightyVersionBuilder, // version::LightyVersionBuilder
};
```

---

## Import Patterns

### Pattern 1: Prelude (Recommended)

```rust
use lighty_launcher::prelude::*;

// Everything you need is available
```

### Pattern 2: Specific Modules

```rust
use lighty_launcher::{
    auth::{OfflineAuth, Authenticator},
    java::JavaDistribution,
    launch::Launch,
    loaders::Loader,
    version::VersionBuilder,
    core::AppState,
};
```

### Pattern 3: Root + Modules

```rust
use lighty_launcher::{VersionBuilder, Loader, JavaDistribution};
use lighty_launcher::auth::OfflineAuth;
use lighty_launcher::launch::Launch;
```

### Pattern 4: Direct Crate Access

```rust
// Use underlying crates directly (advanced)
use lighty_auth::OfflineAuth;
use lighty_launch::Launch;
use lighty_version::VersionBuilder;
```

---

## Related Documentation

- [Sequence Diagrams](./sequence-diagrams.md) - Visual flow diagrams
- [Architecture](./architecture.md) - System architecture
- [Examples](./examples.md) - Code examples

### Crate Documentation

- [lighty-core](../crates/core/README.md)
- [lighty-auth](../crates/auth/README.md)
- [lighty-event](../crates/event/README.md)
- [lighty-java](../crates/java/README.md)
- [lighty-launch](../crates/launch/README.md)
- [lighty-loaders](../crates/loaders/README.md)
- [lighty-modsloader](../crates/modsloader/docs/overview.md)
- [lighty-version](../crates/version/README.md)
