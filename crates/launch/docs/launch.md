# Launch Process

## Overview

The launch process orchestrates all steps required to start a Minecraft instance, from metadata fetching to process execution.

## Launch Flow Diagram

```
1. Prepare Metadata
   ├─> Fetch loader metadata (Vanilla/Fabric/Quilt/NeoForge)
   └─> Validate version information

2. Ensure Java Installed
   ├─> Check for Java runtime
   ├─> Download if missing
   └─> Validate Java version

3. Install Dependencies
   ├─> Verify existing files (SHA1 check)
   ├─> Download Libraries
   ├─> Download Natives
   ├─> Download Client JAR
   ├─> Download Assets
   └─> Download Mods (if applicable)

4. Build Arguments
   ├─> Create variable map
   ├─> Apply JVM overrides
   ├─> Apply game overrides
   └─> Substitute placeholders

5. Execute Game
   ├─> Spawn Java process
   ├─> Register instance (PID tracking)
   ├─> Stream console output
   └─> Wait for exit
```

## Step-by-Step Process

### 1. Prepare Metadata

**Purpose**: Fetch version metadata from the loader API

```rust
let metadata = builder.get_metadata().await?;
```

**What happens**:
- Dispatches to correct loader implementation (Vanilla, Fabric, Quilt, NeoForge)
- Fetches version manifest from official APIs
- Parses JSON into `VersionMetaData` structure
- Emits `LoaderEvent::FetchingData` and `LoaderEvent::DataFetched` (with events feature)

**Example metadata structure**:
```json
{
  "id": "1.21.1",
  "type": "release",
  "mainClass": "net.minecraft.client.main.Main",
  "libraries": [...],
  "assets": {...},
  "javaVersion": { "majorVersion": 21 }
}
```

### 2. Ensure Java Installed

**Purpose**: Verify correct Java version is available

```rust
let java_path = ensure_java_installed(
    version,
    version_data,
    &java_distribution,
    event_bus,
).await?;
```

**What happens**:
- Extracts required Java version from metadata (`version.java_version.major_version`)
- Searches for existing Java installation in `java_dirs()`
- If not found:
  - Downloads Java from selected distribution (Temurin, Zulu, etc.)
  - Extracts to `java_dirs()/jre/java-{version}/`
  - Validates installation
- Returns path to `java` or `java.exe` executable

**Supported distributions**: Temurin, Zulu, Graal, Corretto

**Events emitted** (with events feature):
- `JavaEvent::JavaNotFound` - When Java needs to be downloaded
- `JavaEvent::JavaDownloading` - Download progress
- `JavaEvent::JavaAlreadyInstalled` - Java already exists

### 3. Install Dependencies

**Purpose**: Download all game files in parallel

See [Installation Documentation](./installation.md) for detailed information.

**High-level flow**:
```rust
version.install(version_data, event_bus).await?;
```

**Phases**:

#### Phase 1: Verification (SHA1 Check)
- Scan existing files
- Compare SHA1 hashes with metadata
- Build task lists for missing/outdated files

#### Phase 2: Decision
- If all files valid → Skip to natives extraction
- Otherwise → Proceed to download

#### Phase 3: Parallel Download
```rust
tokio::try_join!(
    libraries::download_libraries(library_tasks, event_bus),
    natives::download_and_extract_natives(native_download_tasks, native_extract_paths, event_bus),
    mods::download_mods(mod_tasks, event_bus),
    resourcepacks::download_resourcepacks(resourcepack_tasks, resourcepack_bytes, event_bus),
    shaderpacks::download_shaderpacks(shaderpack_tasks, shaderpack_bytes, event_bus),
    datapacks::download_datapacks(datapack_tasks, datapack_bytes, event_bus),
    client::download_client(client_task, event_bus),
    assets::download_assets(asset_tasks, event_bus),
)?;
```

**Directory structure created**:
```
game_dirs/
├── libraries/          # Java JAR libraries
├── natives/            # Platform-specific binaries (LWJGL, etc.)
├── assets/objects/     # Game assets (textures, sounds, etc.)
├── mods/               # Mod files (Fabric/Quilt/NeoForge/Forge)
├── resourcepacks/      # Resource packs routed by mod source
├── shaderpacks/        # Shader packs routed by mod source
├── datapacks/          # Datapacks routed by mod source
└── versions/           # Minecraft client JAR
```

### 4. Build Arguments

**Purpose**: Construct complete command-line arguments

See [Arguments Documentation](./arguments.md) for detailed information.

```rust
let arguments = builder.build_arguments(
    version,
    username,
    uuid,
    arg_overrides,
    arg_removals,
    jvm_overrides,
    jvm_removals,
    raw_args,
);
```

**Argument categories**:
1. **JVM Arguments**: Memory, garbage collection, system properties
2. **Main Class**: `net.minecraft.client.main.Main`
3. **Game Arguments**: Username, directories, authentication
4. **Raw Arguments**: Custom arguments passed directly

#### Token routing (`--accessToken`)

The access token is the most sensitive piece of data the launch
pipeline handles, so the resolution is deliberately narrow. At
`crates/launch/src/arguments/arguments.rs:229` the variable-map
builder runs the following sequence:

1. If `profile.token_handle` is `Some` **and** the `keyring` feature is
   active on `lighty-launch`, read the token from the OS keychain via
   `TokenHandle::read()` — the token never sat in `UserProfile` heap
   memory between authentication and launch.
2. Otherwise, clone the in-memory `SecretString` stored in
   `profile.access_token`.
3. `ExposeSecret::expose_secret(&secret)` is called **exactly once**,
   at the moment the value is inserted into the argv placeholder map
   for `--accessToken`. No copy of the plaintext is kept past that
   call.

```rust
let token_secret: Option<SecretString> = profile.and_then(|p| {
    #[cfg(feature = "keyring")]
    if let Some(h) = &p.token_handle {
        return h.read().ok();
    }
    p.access_token.clone()
});
let access_token = token_secret
    .as_ref()
    .map(|s| s.expose_secret())
    .unwrap_or(DEFAULT_ACCESS_TOKEN);
```

When no profile is supplied (offline / dry-run callers), the legacy
hardcoded default is used. See `AUTH_SECRETS.md` at the repo root for
the full threat model behind this design.

**Feature flag**: `lighty-launch` exposes a `keyring` feature that
forwards to `lighty-auth/keyring`. Enable it on consumers that opt in
to `MicrosoftAuth::with_keyring(...)` / `AzuriomAuth::with_keyring(...)`
so the argv builder above can read from the keychain:

```toml
[dependencies]
lighty-launch = { version = "...", features = ["events", "keyring"] }
```

**Example result**:
```bash
java \
  -Djava.library.path=/tmp/natives-xxxxx \
  -Dminecraft.launcher.brand=MyLauncher \
  -Xmx4G -Xms2G -XX:+UseG1GC \
  -cp /path/lib1.jar:/path/lib2.jar:... \
  net.minecraft.client.main.Main \
  --username Player123 \
  --version 1.21.1 \
  --gameDir /home/user/.local/share/MyLauncher/instance \
  --assetsDir /home/user/.local/share/MyLauncher/assets \
  --uuid 550e8400-... \
  --accessToken 0
```

### 5. Execute Game

**Purpose**: Spawn the Java process and track it

```rust
let child = java_runtime.execute(arguments, game_dir).await?;
let pid = child.id().ok_or(InstallerError::NoPid)?;
```

**What happens**:

#### 5.1. Spawn Process
```rust
let child = Command::new(java_path)
    .args(arguments)
    .current_dir(game_dir)
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()?;
```

#### 5.2. Register Instance
```rust
let instance = GameInstance {
    pid,
    instance_name: builder.name().to_string(),
    version: format!("{}-{}", minecraft_version, loader_version),
    username: username.to_string(),
    game_dir: game_dir.to_path_buf(),
    started_at: SystemTime::now(),
};

INSTANCE_MANAGER.register_instance(instance).await;
```

#### 5.3. Emit Launch Event
```rust
EVENT_BUS.emit(Event::InstanceLaunched(InstanceLaunchedEvent {
    pid,
    instance_name,
    version,
    username,
    timestamp: SystemTime::now(),
}));
```

#### 5.4. Stream Console Output
```rust
tokio::spawn(handle_console_streams(pid, instance_name, child));
```

**Console streaming** (asynchronous):
- Spawns separate tasks for stdout and stderr
- Emits `ConsoleOutputEvent` for each line
- Waits for process exit
- Emits `InstanceExitedEvent` on termination
- Unregisters instance from manager

## Launch Trait

The `Launch` trait provides the entry point:

```rust
pub trait Launch {
    fn launch<'a>(
        &'a mut self,
        profile: &'a UserProfile,
        java_distribution: JavaDistribution
    ) -> LaunchBuilder<'a, Self>
    where
        Self: Sized;
}
```

**Automatically implemented** for any type implementing:
- `VersionInfo<LoaderType = Loader>`
- `LoaderExtensions`
- `Arguments`
- `Installer`

## LaunchBuilder API

The `LaunchBuilder` provides a fluent API:

```rust
pub struct LaunchBuilder<'a, T> {
    version: &'a mut T,
    profile: &'a UserProfile,
    java_distribution: JavaDistribution,
    jvm_overrides: HashMap<String, String>,
    jvm_removals: HashSet<String>,
    arg_overrides: HashMap<String, String>,
    arg_removals: HashSet<String>,
    raw_args: Vec<String>,
    event_bus: Option<&'a EventBus>,
}
```

**Methods**:
- `with_jvm_options()` → Configure JVM options
- `with_arguments()` → Configure game arguments
- `with_event_bus(&bus)` → Set event bus for progress tracking
- `run()` → Execute the launch

## Complete Example

```rust
use lighty_core::AppState;
use lighty_launcher::prelude::*;
use lighty_auth::{offline::OfflineAuth, Authenticator};
use lighty_java::JavaDistribution;
use lighty_launch::InstanceControl;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize AppState
    AppState::init("MyLauncher")?;

    // 2. Create instance
    let mut instance = VersionBuilder::new(
        "my-game",
        Loader::Fabric,
        "0.16.9",
        "1.21.1",
    );

    // 3. Authenticate
    let mut auth = OfflineAuth::new("Player123");

    #[cfg(not(feature = "events"))]
    let profile = auth.authenticate().await?;

    // 4. Launch with custom options
    instance.launch(&profile, JavaDistribution::Temurin)
        .with_jvm_options()
            .set("Xmx", "4G")
            .set("Xms", "2G")
            .set("XX:+UseG1GC", "")
            .done()
        .with_arguments()
            .set("width", "1920")
            .set("height", "1080")
            .done()
        .run()
        .await?;

    println!("Game launched!");

    // 5. Get PID
    if let Some(pid) = instance.get_pid() {
        println!("Running with PID: {}", pid);
    }

    Ok(())
}
```

## With Event Tracking

```rust
use lighty_event::{EventBus, Event, LaunchEvent, ModloaderEvent};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("MyLauncher")?;

    let event_bus = EventBus::new(1000);

    // Subscribe to events
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
                Event::Modloader(ModloaderEvent::ResolveCompleted { total_mods }) => {
                    println!("Resolved {} mods", total_mods);
                }
                Event::Modloader(ModloaderEvent::ResourcePacksInstalled { count, bytes }) => {
                    println!("ResourcePacks: {} files / {} bytes", count, bytes);
                }
                Event::InstanceLaunched(e) => {
                    println!("Launched: {} (PID: {})", e.instance_name, e.pid);
                }
                Event::ConsoleOutput(e) => {
                    println!("[{}] {}", e.pid, e.line);
                }
                Event::InstanceExited(e) => {
                    println!("Exited: PID {} (code: {:?})", e.pid, e.exit_code);
                }
                _ => {}
            }
        }
    });

    let mut instance = VersionBuilder::new(/*...*/);
    let mut auth = OfflineAuth::new("Player");
    let profile = auth.authenticate().await?;

    instance.launch(&profile, JavaDistribution::Temurin)
        .with_event_bus(&event_bus)
        .run()
        .await?;

    Ok(())
}
```

## Error Handling

### InstallerError Types

```rust
pub enum InstallerError {
    DownloadFailed(String),
    VerificationFailed(String),
    ExtractionFailed(String),
    InvalidMetadata,
    NoPid,
    IOError(std::io::Error),
}
```

**Example**:
```rust
match instance.launch(&profile, JavaDistribution::Temurin).run().await {
    Ok(_) => println!("Launched"),
    Err(InstallerError::DownloadFailed(msg)) => {
        eprintln!("Download failed: {}", msg);
    }
    Err(InstallerError::NoPid) => {
        eprintln!("Process started but PID unavailable");
    }
    Err(e) => {
        eprintln!("Launch error: {}", e);
    }
}
```

## Platform Differences

### Windows
- Java executable: `java.exe`
- Classpath separator: `;`
- Process kill: `taskkill /PID {pid} /F`
- Natives: `lwjgl-*-natives-windows.jar`

### Linux
- Java executable: `java`
- Classpath separator: `:`
- Process kill: `kill -SIGTERM {pid}`
- Natives: `lwjgl-*-natives-linux.jar`

### macOS
- Java executable: `java`
- Classpath separator: `:`
- Process kill: `kill -SIGTERM {pid}`
- Natives: `lwjgl-*-natives-macos.jar`

## Performance Optimization

### Parallel Downloads
All 8 buckets download concurrently using `tokio::try_join!`:
- Libraries (~200 files)
- Natives (~10 files)
- Assets (~5000 files)
- Client JAR (1 file)
- Mods (variable, routed to `mods/`)
- Resource packs (variable, routed to `resourcepacks/`)
- Shader packs (variable, routed to `shaderpacks/`)
- Datapacks (variable, routed to `datapacks/`)

### SHA1 Verification
Files are verified before download:
- Existing files with matching SHA1 are skipped
- Only missing/outdated files are downloaded
- Saves bandwidth and time on subsequent launches

### Native Libraries Extraction
- Natives are extracted to a temporary directory on each launch
- Ensures clean state and proper platform isolation
- Directory: `{temp}/natives-{timestamp}/`

## Related Documentation

- [Arguments](./arguments.md) - Detailed argument system
- [Installation](./installation.md) - Installation process details
- [Instance Control](./instance-control.md) - Process management
- [Events](./events.md) - Event system reference
- [How to Use](./how-to-use.md) - Practical examples
- [Exports](./exports.md) - Module exports
