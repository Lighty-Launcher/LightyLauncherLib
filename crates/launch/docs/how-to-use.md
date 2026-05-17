# How to Use lighty-launch

## Basic Usage

### Step 1: Initialize AppState

```rust
use lighty_core::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("MyLauncher")?;

    Ok(())
}
```

### Step 2: Create Instance

```rust
use lighty_launcher::prelude::*;

let mut instance = VersionBuilder::new(
    "my-instance",
    Loader::Fabric,
    "0.16.9",
    "1.21.1",
);
```

### Step 3: Authenticate

```rust
use lighty_auth::{offline::OfflineAuth, Authenticator};

let mut auth = OfflineAuth::new("Player123");

#[cfg(not(feature = "events"))]
let profile = auth.authenticate().await?;

#[cfg(feature = "events"))]
let profile = auth.authenticate(None).await?;
```

### Step 4: Launch

```rust
use lighty_java::JavaDistribution;
use lighty_launch::InstanceControl; // IMPORTANT: Import the trait

instance.launch(&profile, JavaDistribution::Temurin)
    .run()
    .await?;

println!("Game launched!");
```

## Launch with Custom JVM Options

```rust
use lighty_launch::InstanceControl;

instance.launch(&profile, JavaDistribution::Temurin)
    .with_jvm_options()
        .set("Xmx", "4G")           // Maximum memory
        .set("Xms", "2G")           // Initial memory
        .set("XX:+UseG1GC", "")     // Use G1 garbage collector
        .done()
    .run()
    .await?;
```

**Common JVM Options**:
- `Xmx` - Maximum heap size (e.g., "4G", "8G")
- `Xms` - Initial heap size (e.g., "2G")
- `XX:+UseG1GC` - Use G1 garbage collector
- `XX:+UnlockExperimentalVMOptions` - Enable experimental features
- `XX:MaxGCPauseMillis` - Target max GC pause time

## Launch with Custom Game Options

```rust
instance.launch(&profile, JavaDistribution::Temurin)
    .with_game_options()
        .set("width", "1920")
        .set("height", "1080")
        .set("fullscreen", "true")
        .done()
    .run()
    .await?;
```

**Common Game Options**:
- `width` / `height` - Window dimensions
- `fullscreen` - Fullscreen mode
- `quickPlayPath` - Quick play server path
- `quickPlaySingleplayer` - Quick play singleplayer world
- `quickPlayMultiplayer` - Quick play multiplayer server

## Instance Management

### Get Running Instance PID

```rust
use lighty_launch::InstanceControl;

if let Some(pid) = instance.get_pid() {
    println!("Instance running with PID: {}", pid);
} else {
    println!("Instance not running");
}
```

### Get All PIDs (Multiple Instances)

```rust
let pids = instance.get_pids();

if pids.is_empty() {
    println!("No instances running");
} else {
    println!("Running instances: {:?}", pids);
}
```

### Close Instance

```rust
if let Some(pid) = instance.get_pid() {
    instance.close_instance(pid).await?;
    println!("Instance closed");
}
```

### Delete Instance

```rust
// Delete instance from disk (only if not running)
instance.delete_instance().await?;
println!("Instance deleted");
```

**Note**: `delete_instance()` will fail if the instance is running. Close it first.

## Instance Size Calculation

```rust
use lighty_launch::InstanceControl;

// Get metadata
let metadata = instance.get_metadata().await?;

// Calculate size
let size = instance.size_of_instance(&metadata);

println!("Libraries: {} MB", size.libraries / 1_000_000);
println!("Client JAR: {} MB", size.client / 1_000_000);
println!("Assets: {} MB", size.assets / 1_000_000);
println!("Mods: {} MB", size.mods / 1_000_000);
println!("Natives: {} MB", size.natives / 1_000_000);
println!("Total: {} MB ({:.2} GB)", size.total / 1_000_000, size.total_gb());

// Or use formatted strings
use lighty_loaders::types::InstanceSize;
println!("Total: {}", InstanceSize::format(size.total));
```

## With Events

Track launch progress with events:

```rust
use lighty_event::{EventBus, Event, LaunchEvent, ModloaderEvent};
use lighty_launch::InstanceControl;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("MyLauncher")?;

    // Create event bus
    let event_bus = EventBus::new(1000);
    let mut receiver = event_bus.subscribe();

    // Spawn event listener
    tokio::spawn(async move {
        while let Ok(event) = receiver.next().await {
            match event {
                Event::Launch(LaunchEvent::InstallStarted { version, total_bytes }) => {
                    println!("Installing {} ({} bytes)", version, total_bytes);
                }
                Event::Launch(LaunchEvent::InstallProgress { bytes }) => {
                    // Accumulate against `total_bytes` from InstallStarted to
                    // drive a progress bar; the installer emits one of these
                    // per chunk written to disk across all 8 buckets.
                    println!("+{} bytes downloaded", bytes);
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
                Event::Modloader(ModloaderEvent::ShaderPacksInstalled { count, bytes }) => {
                    println!("ShaderPacks: {} files / {} bytes", count, bytes);
                }
                Event::Modloader(ModloaderEvent::DatapacksInstalled { count, bytes }) => {
                    println!("Datapacks: {} files / {} bytes", count, bytes);
                }
                Event::Launch(LaunchEvent::Launched { version, pid }) => {
                    println!("Launched {} with PID {}", version, pid);
                }
                Event::Launch(LaunchEvent::ProcessOutput { pid, line, .. }) => {
                    print!("[{}] {}", pid, line);
                }
                Event::Launch(LaunchEvent::ProcessExited { pid, exit_code }) => {
                    println!("Instance {} exited with code: {}", pid, exit_code);
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
    let profile = auth.authenticate(None).await?;

    // Launch with events
    instance.launch(&profile, JavaDistribution::Temurin)
        .run()
        .await?;

    // Keep running to see console output
    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;

    Ok(())
}
```

## Installation Only (No Launch)

Sometimes you want to download assets without launching:

```rust
use lighty_launch::installer::Installer;

// Get metadata
let metadata = instance.get_metadata().await?;

// Install assets, libraries, natives, mods
Installer.install(&instance, &metadata).await?;

println!("Installation complete!");
```

## Complete Launch Flow

```rust
use lighty_core::AppState;
use lighty_launcher::prelude::*;
use lighty_java::JavaDistribution;
use lighty_launch::InstanceControl;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize AppState
    AppState::init("MyLauncher")?;

    // 2. Create instance
    let mut instance = VersionBuilder::new(
        "my-modpack",
        Loader::Fabric,
        "0.16.9",
        "1.21.1",
    );

    // 3. Authenticate
    let mut auth = MicrosoftAuth::new("your-client-id");
    auth.set_device_code_callback(|code, url| {
        println!("Visit: {}", url);
        println!("Code: {}", code);
    });

    #[cfg(not(feature = "events"))]
    let profile = auth.authenticate().await?;

    #[cfg(feature = "events")]
    let profile = auth.authenticate(None).await?;

    // 4. Calculate instance size before launching
    let metadata = instance.get_metadata().await?;
    let size = instance.size_of_instance(&metadata);
    println!("Instance size: {:.2} GB", size.total_gb());

    // 5. Launch with custom options
    instance.launch(&profile, JavaDistribution::Temurin)
        .with_jvm_options()
            .set("Xmx", "6G")
            .set("Xms", "2G")
            .done()
        .with_game_options()
            .set("width", "1920")
            .set("height", "1080")
            .done()
        .run()
        .await?;

    println!("Game launched!");

    // 6. Get PID
    if let Some(pid) = instance.get_pid() {
        println!("Running with PID: {}", pid);
    }

    // 7. Wait a bit, then close
    tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;

    if let Some(pid) = instance.get_pid() {
        instance.close_instance(pid).await?;
        println!("Game closed");
    }

    Ok(())
}
```

## Error Handling

```rust
use lighty_launch::errors::{InstallerError, InstanceError};

// Launch errors
match instance.launch(&profile, JavaDistribution::Temurin).run().await {
    Ok(_) => println!("Launched successfully"),
    Err(e) => {
        eprintln!("Launch failed: {}", e);
        // Handle specific errors
        match e.downcast_ref::<InstallerError>() {
            Some(InstallerError::DownloadFailed(url)) => {
                eprintln!("Failed to download: {}", url);
            }
            Some(InstallerError::VerificationFailed(file)) => {
                eprintln!("Hash verification failed: {}", file);
            }
            _ => {}
        }
    }
}

// Instance errors
match instance.delete_instance().await {
    Ok(_) => println!("Deleted"),
    Err(InstanceError::InstanceRunning) => {
        eprintln!("Cannot delete: instance is running");
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## Feature Flags

```toml
[dependencies]
lighty-launch = { version = "26.5.1", features = ["events"] }
```

Available features:
- `events` - Enables LaunchEvent emission (requires lighty-event)

## Exports

**In lighty_launch**:
```rust
use lighty_launch::{
    // Builder
    LaunchBuilder,
    LaunchConfig,

    // Trait (MUST import to use methods)
    InstanceControl,

    // Installer
    installer::Installer,

    // Errors
    errors::{InstallerError, InstallerResult, InstanceError, InstanceResult},

    // Arguments
    arguments::Arguments,
};
```

**In lighty_launcher**:
```rust
use lighty_launcher::launch::{
    LaunchBuilder,
    InstanceControl,
    // ... etc
};
```

## Related Documentation

- [Overview](./overview.md) - Architecture and design
- [Events](./events.md) - LaunchEvent types
- [Exports](./exports.md) - Complete export reference
- [Launch Process](./launch.md) - Detailed launch workflow
- [Installation](./installation.md) - Asset and library installation details
- [Arguments](./arguments.md) - JVM and game argument generation
- [Instance Control](./instance-control.md) - Instance management details

## Related Crates

- **[lighty-core](../../core/README.md)** - AppState and utilities
- **[lighty-java](../../java/README.md)** - Java runtime management
- **[lighty-version](../../version/README.md)** - VersionBuilder
- **[lighty-loaders](../../loaders/README.md)** - Loader metadata
- **[lighty-auth](../../auth/README.md)** - User authentication
- **[lighty-event](../../event/README.md)** - Event system
