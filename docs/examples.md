# Examples Documentation

Complete guide to all examples in the `examples/` directory.

## Overview

### Loaders (one per loader, flat under `examples/`)

| Example | Loader | Features |
|---------|--------|----------|
| [vanilla.rs](#vanillars) | Vanilla | Basic launch |
| [fabric.rs](#fabricrs) | Fabric | Basic launch |
| [quilt.rs](#quiltrs) | Quilt | Basic launch |
| [neoforge.rs](#neoforgrs) | NeoForge | MS auth + events + console output |
| [forge.rs](#forgrs) | Forge (modern + legacy via same flag) | MS auth + events + console output |
| [optifine.rs](#optifinrs) | OptiFine | Basic launch |
| [lighty_updater.rs](#lighty_updaterrs) | LightyUpdater | Custom server |

### Auth + persistent "remember me" (under `examples/auth/`)

| Example | Provider | Purpose |
|---------|----------|---------|
| `auth/microsoft.rs` | Microsoft | Silent refresh-token re-auth → fallback device-code, keyring persistence |
| `auth/azuriom.rs` | Azuriom | `verify()` of saved session → fallback credentials, keyring persistence |
| `auth/custom.rs` | Custom backend | Skeleton `Authenticator` impl + the same keyring pattern |

All three use the OS keyring (Linux Secret Service / macOS Keychain /
Windows Credential Manager) via the [`keyring`](https://crates.io/crates/keyring)
crate. The library itself does not depend on `keyring` — persistence
is intentionally left to the consumer.

### Mods (under `examples/mods/`)

Pin individual mods from Modrinth or CurseForge. Both examples now
include a `(slug, Some(version_id))` / `(mod_id, Some(file_id))` pinned
entry alongside the latest-compatible one, with header-docs explaining
where to find the IDs on the Modrinth/CurseForge web UIs.

| Example | Source | Auth required |
|---------|--------|---------------|
| `mods/modrinth.rs` | Modrinth (public API) | No |
| `mods/curseforge.rs` | CurseForge (keyed API) | `CURSEFORGE_API_KEY` env var |

The builder API was renamed in this release:

- `with_modrinth(...)` → `with_modrinth_mods(...)`
- `with_curseforge(...)` → `with_curseforge_mods(...)`

See [`crates/modsloader/docs/mods.md`](../crates/modsloader/docs/mods.md)
for the procedure to find `version_id` / `file_id`.

### Modpacks (under `examples/modpacks/`)

Install a community modpack (mods + config overrides) from one source
in a single call. All three ride on the matching provider feature
(`modrinth` for `.mrpack`, `curseforge` for `.zip`) — there is no
separate `modpack` feature.

| Example | Source | Auth required |
|---------|--------|---------------|
| `modpacks/modrinth_url.rs` | Modrinth `.mrpack` via direct CDN URL | No |
| `modpacks/modrinth_pinned.rs` | Modrinth `.mrpack` via `(project, version_id)` | No |
| `modpacks/curseforge.rs` | CurseForge `.zip` via `(project_id, file_id)` | `CURSEFORGE_API_KEY` env var |

See [`crates/modsloader/docs/modpacks.md`](../crates/modsloader/docs/modpacks.md)
for the manifest formats, Modrinth whitelist, conflict policy on
overrides, and idempotence model.

### Full showcase (under `examples/with_events/`)

| Example | Loader | Features |
|---------|--------|----------|
| [with_events.rs](#with_eventsrs) | Vanilla | Full event-bus tour + instance control |

## Running Examples

### Basic Command

```bash
cargo run --example <example_name> --features <required_features>
```

### With Tracing (Debug Output)

```bash
cargo run --example <example_name> --features <required_features>,tracing
```

### Common Feature Combinations

```bash
# Vanilla with events
cargo run --example vanilla --features vanilla,events

# Fabric with events and tracing
cargo run --example fabric --features fabric,events,tracing

# Complete demo
cargo run --example with_events --features vanilla,events,tracing
```

---

## vanilla.rs

**Purpose**: Basic Vanilla Minecraft launcher with custom JVM options and game arguments

**Loader**: Vanilla
**Features Required**: `vanilla`

### What It Demonstrates

- ✅ AppState initialization
- ✅ Offline authentication
- ✅ VersionBuilder creation
- ✅ Custom JVM options (memory, etc.)
- ✅ Custom game arguments (resolution)
- ✅ Downloader configuration
- ✅ Basic launch flow

### Code Walkthrough

```rust
use lighty_launcher::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize AppState
    AppState::init("LightyLauncher")?;

    // 2. Configure downloader (optional)
    init_downloader_config(DownloaderConfig {
        max_concurrent_downloads: 100,
        max_retries: 5,
        initial_delay_ms: 50,
    });

    // 3. Authenticate
    let mut auth = OfflineAuth::new("Hamadi");
    let profile = auth.authenticate().await?;

    // 4. Create instance
    let mut version = VersionBuilder::new(
        "vanilla-1.7.10",
        Loader::Vanilla,
        "",
        "1.7.10",
    );

    // 5. Launch with custom options
    version.launch(&profile, JavaDistribution::Temurin)
        .with_jvm_options()
            .set("Xmx", "4G")          // Max heap: 4GB
            .set("Xms", "2G")          // Initial heap: 2GB
            .done()
        .with_arguments()
            .set("width", "1920")      // Window width
            .set("height", "1080")     // Window height
            .done()
        .run()
        .await?;

    trace_info!("Launch successful!");
    Ok(())
}
```

### Key Concepts

1. **AppState**: Manages project directories (data, config, cache)
2. **DownloaderConfig**: Configures parallel downloads and retries
3. **JVM Options**: Memory allocation, garbage collection
4. **Game Arguments**: Window resolution, fullscreen, etc.

### Run It

```bash
cargo run --example vanilla --features vanilla
```

---

## fabric.rs

**Purpose**: Fabric mod loader launcher

**Loader**: Fabric
**Features Required**: `fabric`

### What It Demonstrates

- ✅ Fabric loader integration
- ✅ Loader version specification
- ✅ Metadata merging (Vanilla + Fabric)

### Code Highlights

```rust
// Fabric instance with loader version
let mut fabric = VersionBuilder::new(
    "fabric",
    Loader::Fabric,
    "0.17.2",      // Fabric loader version
    "1.21.1",      // Minecraft version
);

fabric.launch(&profile, JavaDistribution::Temurin)
    .run()
    .await?;
```

### How It Works

1. Fetches Vanilla 1.21.1 metadata
2. Fetches Fabric 0.17.2 loader data
3. Merges metadata (adds Fabric libraries, updates main class)
4. Launches with merged metadata

### Run It

```bash
cargo run --example fabric --features fabric
```

---

## quilt.rs

**Purpose**: Quilt mod loader launcher

**Loader**: Quilt
**Features Required**: `quilt`

### What It Demonstrates

- ✅ Quilt loader integration
- ✅ Alternative to Fabric

### Code Highlights

```rust
let mut quilt = VersionBuilder::new(
    "quilt",
    Loader::Quilt,
    "0.27.1",      // Quilt loader version
    "1.21.1",      // Minecraft version
);
```

### Run It

```bash
cargo run --example quilt --features quilt
```

---

## neoforge.rs

**Purpose**: NeoForge mod loader launcher with Microsoft auth and live
console streaming. Covers both eras (`net.neoforged:forge` for MC 1.20.1
and `net.neoforged:neoforge` for MC ≥ 1.20.2).

**Loader**: NeoForge
**Features Required**: `neoforge`, `events`, `tracing`

### What It Demonstrates

- ✅ Microsoft device-code authentication
- ✅ NeoForge processor pipeline
- ✅ Live `[GAME]` / `[GAME ERR]` console output via the event bus

### Code Highlights

```rust
let mut neoforge = VersionBuilder::new(
    "neoforge-1.21.8",
    Loader::NeoForge,
    "21.8.53",   // NeoForge version
    "1.21.8",    // Minecraft version
);
```

### Run It

```bash
cargo run --example neoforge --features neoforge,events,tracing
```

---

## forge.rs

**Purpose**: Forge mod loader launcher with Microsoft auth and live
console streaming. The `forge` feature covers both modern (≥ 1.13) and
legacy (1.5.2 → 1.12.2) — the loader dispatches based on the installer
schema, so the same example works for any version.

**Loader**: Forge
**Features Required**: `forge`, `events`, `tracing`

### What It Demonstrates

- ✅ Microsoft device-code authentication
- ✅ Modern Forge install pipeline (`install_profile.json` + processors)
- ✅ Library merge with classifier-safe dedup (`:universal` + `:client`)
- ✅ Live `[GAME]` / `[GAME ERR]` console output via the event bus
- ✅ Blocking on `Event::InstanceExited` so the runtime survives the JVM

### Modern (≥ 1.13)

```rust
let mut forge = VersionBuilder::new(
    "forge-1.21.8",
    Loader::Forge,
    "58.1.0",   // Forge loader version
    "1.21.8",   // Minecraft version
);
```

### Legacy (1.5.2 → 1.12.2)

Same example, just swap the loader/MC versions — no separate feature
flag and no separate example file:

```rust
let mut forge = VersionBuilder::new(
    "forge-1.12.2",
    Loader::Forge,
    "14.23.5.2860",
    "1.12.2",
);
```

### Run It

```bash
cargo run --example forge --features forge,events,tracing
```

---

## optifine.rs

**Purpose**: OptiFine launcher

**Loader**: OptiFine
**Features Required**: `optifine`

### What It Demonstrates

- ✅ OptiFine integration
- ✅ Performance optimization mod

### Code Highlights

```rust
let mut optifine = VersionBuilder::new(
    "optifine",
    Loader::Optifine,
    "HD_U_I6",     // OptiFine version
    "1.20.1",      // Minecraft version
);
```

### Run It

```bash
cargo run --example optifine --features optifine
```

---

## lighty_updater.rs

**Purpose**: Custom modpack server launcher

**Loader**: LightyUpdater
**Features Required**: `lighty_updater`

### What It Demonstrates

- ✅ Custom server integration
- ✅ LightyVersionBuilder
- ✅ Server-managed modpacks
- ✅ Automatic mod updates

### Code Highlights

```rust
use lighty_launcher::version::LightyVersionBuilder;

// Connect to custom server
let mut modpack = LightyVersionBuilder::new(
    "my-modpack",
    "https://myserver.com/api",  // Server URL
);

modpack.launch(&profile, JavaDistribution::Temurin)
    .run()
    .await?;
```

### How It Works

1. **GET {server_url}/version**
   ```json
   {
       "minecraft_version": "1.21.1",
       "loader": "Fabric",
       "loader_version": "0.17.2",
       "mods": [
           {
               "name": "fabric-api",
               "url": "https://server.com/mods/fabric-api.jar",
               "sha1": "abc123...",
               "enabled": true
           }
       ]
   }
   ```

2. Fetches Vanilla + Loader metadata
3. Adds server mods to metadata
4. Downloads and installs mods
5. Launches game

### Server Setup

See [LightyUpdater Repository](https://github.com/Lighty-Launcher/LightyUpdater) for server implementation.

### Run It

```bash
cargo run --example lighty_updater --features lighty_updater
```

---

## with_events.rs

**Purpose**: Complete demonstration of event system and instance management

**Loader**: Vanilla
**Features Required**: `vanilla`, `events`

### What It Demonstrates

- ✅ Event bus creation and subscription
- ✅ All event types (Auth, Java, Launch, Loader, Core, Instance)
- ✅ Real-time progress tracking
- ✅ Console output streaming
- ✅ Instance lifecycle management
- ✅ Instance size calculation
- ✅ PID tracking
- ✅ Instance control (close, delete)

### Code Walkthrough

```rust
use lighty_launcher::prelude::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize AppState
    AppState::init("MyLauncher")?;

    // 2. Create event bus
    let event_bus = EventBus::new(1000);

    // 3. Spawn event listener
    let mut receiver = event_bus.subscribe();
    tokio::spawn(async move {
        while let Ok(event) = receiver.next().await {
            match event {
                Event::Auth(AuthEvent::AuthenticationSuccess { username, .. }) => {
                    trace_info!("Authenticated as {}", username);
                }
                Event::Launch(LaunchEvent::InstallStarted { total_bytes, .. }) => {
                    trace_info!("Installing: {} MB total", total_bytes / 1_000_000);
                }
                Event::Launch(LaunchEvent::InstallProgress { bytes }) => {
                    // Track download progress
                }
                Event::Java(JavaEvent::JavaDownloadStarted { distribution, version, total_bytes }) => {
                    trace_info!("[Java] Downloading {} {} ({} MB)", distribution, version, total_bytes / 1_000_000);
                }
                Event::InstanceLaunched(e) => {
                    trace_info!("\n[EVENT] Instance '{}' launched", e.instance_name);
                    trace_info!("PID: {}", e.pid);
                }
                Event::ConsoleOutput(e) => {
                    // Stream console output in real-time
                    let prefix = match e.stream {
                        ConsoleStream::Stdout => "[GAME]",
                        ConsoleStream::Stderr => "[ERR ]",
                    };
                    print!("{} {}", prefix, e.line);
                }
                Event::InstanceExited(e) => {
                    trace_info!("\n[EVENT] Instance exited with code: {:?}", e.exit_code);
                }
                _ => {}
            }
        }
    });

    // 4. Authenticate with events
    let mut auth = OfflineAuth::new("Player");
    let profile = auth.authenticate(Some(&event_bus)).await?;

    // 5. Create instance
    let mut instance = VersionBuilder::new(
        "demo-instance",
        Loader::Vanilla,
        "",
        "1.21.1",
    );

    // 6. Calculate instance size
    let metadata = instance.get_metadata().await?;
    let version = match metadata.as_ref() {
        VersionMetaData::Version(v) => v,
        _ => anyhow::bail!("Expected Version metadata"),
    };

    let size = instance.size_of_instance(&version);
    trace_info!("Libraries: {}", InstanceSize::format(size.libraries));
    trace_info!("Client: {}", InstanceSize::format(size.client));
    trace_info!("Assets: {}", InstanceSize::format(size.assets));
    trace_info!("Total: {:.2} GB", size.total_gb());

    // 7. Launch with event bus
    instance.launch(&profile, JavaDistribution::Liberica)
        .with_event_bus(&event_bus)
        .with_jvm_options()
            .set("Xmx", "2G")
            .set("Xms", "1G")
            .done()
        .with_arguments()
            .set("width", "1280")
            .set("height", "720")
            .done()
        .run()
        .await?;

    // 8. Wait for instance to start
    tokio::time::sleep(Duration::from_secs(2)).await;

    // 9. Check running instances
    if let Some(pid) = instance.get_pid() {
        trace_info!("Instance is running with PID: {}", pid);
        let all_pids = instance.get_pids();
        trace_info!("All PIDs for this instance: {:?}", all_pids);
    }

    // 10. Let it run for a while
    tokio::time::sleep(Duration::from_secs(30)).await;

    // 11. Close the instance
    if let Some(pid) = instance.get_pid() {
        instance.close_instance(pid).await?;
        trace_info!("Instance closed successfully");
    }

    // 12. Delete the instance
    tokio::time::sleep(Duration::from_secs(2)).await;
    instance.delete_instance().await?;
    trace_info!("Instance deleted from disk");

    Ok(())
}
```

### Event Types Demonstrated

#### 1. Authentication Events

```rust
Event::Auth(AuthEvent::AuthenticationStarted { provider })
Event::Auth(AuthEvent::AuthenticationSuccess { username, uuid })
Event::Auth(AuthEvent::AuthenticationFailed { error, provider })
```

#### 2. Java Events

```rust
Event::Java(JavaEvent::JavaNotFound { distribution, version })
Event::Java(JavaEvent::JavaAlreadyInstalled { distribution, version, binary_path })
Event::Java(JavaEvent::JavaDownloadStarted { distribution, version, total_bytes })
Event::Java(JavaEvent::JavaDownloadProgress { bytes })
Event::Java(JavaEvent::JavaDownloadCompleted { distribution, version })
Event::Java(JavaEvent::JavaExtractionStarted { distribution, version })
Event::Java(JavaEvent::JavaExtractionCompleted { distribution, version, binary_path })
```

#### 3. Launch Events

```rust
Event::Launch(LaunchEvent::IsInstalled { version })
Event::Launch(LaunchEvent::InstallStarted { version, total_bytes })
Event::Launch(LaunchEvent::InstallProgress { bytes })
Event::Launch(LaunchEvent::InstallCompleted { version, total_bytes })
Event::Launch(LaunchEvent::Launching { version })
Event::Launch(LaunchEvent::Launched { version, pid })
Event::Launch(LaunchEvent::NotLaunched { version, error })
Event::Launch(LaunchEvent::ProcessOutput { pid, stream, line })
Event::Launch(LaunchEvent::ProcessExited { pid, exit_code })
```

#### 4. Loader Events

```rust
Event::Loader(LoaderEvent::FetchingData { loader, minecraft_version, loader_version })
Event::Loader(LoaderEvent::DataFetched { loader, minecraft_version, loader_version })
Event::Loader(LoaderEvent::ManifestCached { loader })
Event::Loader(LoaderEvent::MergingLoaderData { base_loader, overlay_loader })
Event::Loader(LoaderEvent::DataMerged { base_loader, overlay_loader })
```

#### 4b. Modloader Events

Dependency resolution, modpack pipeline, and per-bucket install
summaries — split out of `LaunchEvent` into its own enum.

```rust
Event::Modloader(ModloaderEvent::ResolveStarted { request_count })
Event::Modloader(ModloaderEvent::ResolveFetching { source, identifier })
Event::Modloader(ModloaderEvent::ResolveDependency { parent, dependency })
Event::Modloader(ModloaderEvent::ResolveCompleted { total_mods })
Event::Modloader(ModloaderEvent::ModpackResolveStart { source })
Event::Modloader(ModloaderEvent::ModpackArchiveDownloaded { sha1, bytes })
Event::Modloader(ModloaderEvent::ModpackOverridesExtracted { count })
Event::Modloader(ModloaderEvent::ModpackInstalled { name, mods_count })
Event::Modloader(ModloaderEvent::ResourcePacksInstalled { count, bytes })
Event::Modloader(ModloaderEvent::ShaderPacksInstalled { count, bytes })
Event::Modloader(ModloaderEvent::DatapacksInstalled { count, bytes })
```

#### 5. Core Events

```rust
Event::Core(CoreEvent::ExtractionStarted { archive_type, source, destination, file_count })
Event::Core(CoreEvent::ExtractionProgress { files_extracted, total_files })
Event::Core(CoreEvent::ExtractionCompleted { archive_type, files_extracted })
```

#### 6. Instance Events

```rust
Event::InstanceLaunched(InstanceLaunchedEvent {
    pid,
    instance_name,
    version,
    username,
    timestamp
})

Event::ConsoleOutput(ConsoleOutputEvent {
    pid,
    instance_name,
    stream,  // Stdout or Stderr
    line,
    timestamp
})

Event::InstanceExited(InstanceExitedEvent {
    pid,
    instance_name,
    exit_code,
    timestamp
})

Event::InstanceDeleted(InstanceDeletedEvent {
    instance_name,
    timestamp
})
```

### Instance Control Operations

```rust
// Get PID
let pid = instance.get_pid();

// Get all PIDs (supports multiple instances)
let pids = instance.get_pids();

// Close instance
instance.close_instance(pid).await?;

// Delete instance (must not be running)
instance.delete_instance().await?;

// Calculate size
let size = instance.size_of_instance(&version);
println!("Total: {:.2} GB", size.total_gb());
```

### Run It

```bash
cargo run --example with_events --features vanilla,events,tracing
```

**Expected Output**:

```
=== Events and Instance Management Example ===

Step 1: Authenticating...
Authenticated as: Player

Step 2: Creating Vanilla instance...
Instance created

Step 3: Calculating instance size...
Libraries: 52.3 MB
Client: 24.2 MB
Assets: 284.7 MB
Total: 361.2 MB (0.35 GB)

Step 4: Launching instance...
[Loader] Fetching Vanilla data for Minecraft 1.21.1
[Loader] Vanilla data fetched successfully
[Java] Temurin 21 already installed
1.21.1 is already installed and up-to-date!

[EVENT] Instance 'demo-instance' launched
PID: 12345
Version: 1.21.1-
Player: Player

[GAME] [14:52:31] [Render thread/INFO]: Setting user: Player
[GAME] [14:52:31] [Render thread/INFO]: Backend library: LWJGL version 3.3.3
[GAME] [14:52:32] [Render thread/INFO]: Reloading ResourceManager...

Step 5: Checking running instances...
Instance is running with PID: 12345
All PIDs for this instance: [12345]

Step 6: Instance running... (waiting 30 seconds)
Console output is being streamed above

Step 7: Closing instance...
Instance closed successfully

Step 8: Deleting instance...
Instance deleted from disk
Example completed successfully
```

---

## Common Patterns

### Pattern 1: Basic Launch

```rust
use lighty_launcher::prelude::*;

AppState::init("MyLauncher")?;

let mut auth = OfflineAuth::new("Player");
let profile = auth.authenticate().await?;

let mut instance = VersionBuilder::new(
    "my-instance",
    Loader::Vanilla,
    "",
    "1.21.1",
);

instance.launch(&profile, JavaDistribution::Temurin)
    .run()
    .await?;
```

### Pattern 2: With Custom Options

```rust
instance.launch(&profile, JavaDistribution::Temurin)
    .with_jvm_options()
        .set("Xmx", "4G")
        .set("Xms", "2G")
        .done()
    .with_arguments()
        .set("width", "1920")
        .set("height", "1080")
        .done()
    .run()
    .await?;
```

### Pattern 3: With Events

```rust
let event_bus = EventBus::new(1000);

// Spawn listener
let mut receiver = event_bus.subscribe();
tokio::spawn(async move {
    while let Ok(event) = receiver.next().await {
        // Handle events
    }
});

// Launch with events
instance.launch(&profile, JavaDistribution::Temurin)
    .with_event_bus(&event_bus)
    .run()
    .await?;
```

### Pattern 4: Instance Management

```rust
use lighty_launch::InstanceControl;  // Must import!

// Launch
instance.launch(&profile, JavaDistribution::Temurin).run().await?;

// Get PID
if let Some(pid) = instance.get_pid() {
    println!("Running: {}", pid);

    // Close
    instance.close_instance(pid).await?;
}

// Delete
instance.delete_instance().await?;
```

## Troubleshooting

### Error: "Failed to fetch metadata"

**Solution**: Check internet connection and loader availability

### Error: "Java not found"

**Solution**: Ensure Java distribution is supported for the Minecraft version

### Error: "Instance is running"

**Solution**: Close instance before deleting:

```rust
for pid in instance.get_pids() {
    instance.close_instance(pid).await?;
}
instance.delete_instance().await?;
```

## Related Documentation

- [Sequence Diagrams](./sequence-diagrams.md) - Visual flow diagrams
- [Re-exports](./reexports.md) - API reference
- [Architecture](./architecture.md) - System architecture
- [Launch Process](../crates/launch/docs/launch.md) - Detailed launch flow
- [Events](../crates/launch/docs/events.md) - Event types reference
- [Instance Control](../crates/launch/docs/instance-control.md) - Process management
