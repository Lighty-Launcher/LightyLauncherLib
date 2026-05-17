# lighty-launch

Minecraft launch system for [LightyLauncher](https://crates.io/crates/lighty-launcher).

## Overview

**Version**: 26.5.1
**Part of**: [LightyLauncher](https://crates.io/crates/lighty-launcher)

`lighty-launch` handles the complete Minecraft launch process, from downloading assets to managing running instances.

## Features

- **Game Launching** - Launch Minecraft with optimized JVM arguments
- **Asset Installation** - Download and install game assets, libraries, and natives
- **Process Management** - Manage Minecraft process lifecycle
- **Instance Control** - Track, monitor, and control running instances
- **Console Streaming** - Real-time console output via events
- **Instance Size Calculation** - Calculate disk space usage
- **Mod Support** - Automatic mod installation and management

## Quick Start

```toml
[dependencies]
lighty-launch = "26.5.1"
```

### Basic Launch

```rust
use lighty_core::AppState;
use lighty_launcher::prelude::*;
use lighty_java::JavaDistribution;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize app state
    AppState::init("MyLauncher")?;

    // Create instance
    let mut instance = VersionBuilder::new(
        "my-instance",
        Loader::Fabric,
        "0.16.9",
        "1.21.1",
    );

    // Authenticate
    let mut auth = OfflineAuth::new("Player123");

    #[cfg(not(feature = "events"))]
    let profile = auth.authenticate().await?;

    #[cfg(feature = "events")]
    let profile = auth.authenticate(None).await?;

    // Launch the game
    instance.launch(&profile, JavaDistribution::Temurin)
        .run()
        .await?;

    Ok(())
}
```

### With Custom JVM Options

```rust
use lighty_launch::InstanceControl; // Import trait

// Launch with custom JVM settings
instance.launch(&profile, JavaDistribution::Temurin)
    .with_jvm_options()
        .set("Xmx", "4G")      // Max memory
        .set("Xms", "2G")      // Initial memory
        .done()
    .run()
    .await?;
```

### Instance Control

```rust
use lighty_launch::InstanceControl; // Import trait

// Get running instance PID
if let Some(pid) = instance.get_pid() {
    println!("Running with PID: {}", pid);

    // Kill the instance
    instance.close_instance(pid).await?;
}

// Calculate instance size
let metadata = instance.get_metadata().await?;
let size = instance.size_of_instance(&metadata);
println!("Total: {} MB", size.total / 1_000_000);

// Delete instance
instance.delete_instance().await?;
```

## Core Components

| Component | Description |
|-----------|-------------|
| **LaunchBuilder** | Fluent API for configuring game launch |
| **Installer** | Downloads assets, libraries, natives, and mods |
| **InstanceControl** | Trait for managing running instances |
| **Arguments** | JVM and game argument generation |
| **Console Streaming** | Real-time console output (with events feature) |

## Documentation

📚 **[Complete Documentation](./docs)**

| Guide | Description |
|-------|-------------|
| [How to Use](./docs/how-to-use.md) | Practical launch guide with examples |
| [Overview](./docs/overview.md) | Architecture and design |
| [Exports](./docs/exports.md) | Complete export reference |
| [Events](./docs/events.md) | LaunchEvent types |
| [Launch Process](./docs/launch.md) | Launch workflow details |
| [Installation](./docs/installation.md) | Asset and library installation |
| [Arguments](./docs/arguments.md) | JVM and game arguments |
| [Instance Control](./docs/instance-control.md) | Managing instances |

## Related Crates

- **[lighty-launcher](../../../README.md)** - Main package
- **[lighty-event](../event/README.md)** - Event system (for LaunchEvent)
- **[lighty-core](../core/README.md)** - AppState and utilities
- **[lighty-java](../java/README.md)** - Java runtime management
- **[lighty-version](../version/README.md)** - VersionBuilder
- **[lighty-loaders](../loaders/README.md)** - Loader metadata
- **[lighty-auth](../auth/README.md)** - UserProfile for launching

## License

MIT
