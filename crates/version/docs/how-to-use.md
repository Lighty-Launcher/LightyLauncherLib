# How to Use lighty-version

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

### Step 2: Create VersionBuilder

```rust
use lighty_version::VersionBuilder;
use lighty_loaders::types::Loader;

// Vanilla instance
let vanilla = VersionBuilder::new(
    "vanilla-1.21",
    Loader::Vanilla,
    "",              // No loader version for Vanilla
    "1.21.1",
);

// Fabric instance
let fabric = VersionBuilder::new(
    "fabric-1.21",
    Loader::Fabric,
    "0.16.9",        // Fabric loader version
    "1.21.1",
);

// Quilt instance
let quilt = VersionBuilder::new(
    "quilt-1.21",
    Loader::Quilt,
    "0.27.1",        // Quilt loader version
    "1.21.1",
);

// NeoForge instance
let neoforge = VersionBuilder::new(
    "neoforge-1.21",
    Loader::NeoForge,
    "21.1.80",       // NeoForge version
    "1.21.1",
);
```

## VersionBuilder Features

### Get Version Information

```rust
use lighty_loaders::types::VersionInfo;

println!("Name: {}", vanilla.name());
println!("Loader: {:?}", vanilla.loader());
println!("Loader version: {}", vanilla.loader_version());
println!("Minecraft version: {}", vanilla.minecraft_version());
println!("Game directory: {}", vanilla.game_dirs().display());
println!("Java directory: {}", vanilla.java_dirs().display());
println!("Full ID: {}", vanilla.full_identifier());
```

**Output**:
```
Name: vanilla-1.21
Loader: Vanilla
Loader version:
Minecraft version: 1.21.1
Game directory: /home/user/.local/share/MyLauncher/vanilla-1.21
Java directory: /home/user/.config/MyLauncher/jre
Full ID: vanilla-1.21-1.21.1-
```

### Custom Directories

```rust
use std::path::PathBuf;

let instance = VersionBuilder::new(
    "custom",
    Loader::Fabric,
    "0.16.9",
    "1.21.1",
)
.with_custom_java_dir(PathBuf::from("/usr/lib/jvm/java-21"));

println!("Java dir: {}", instance.java_dirs().display());
// Output: Java dir: /usr/lib/jvm/java-21

// To relocate the runtime (mods/saves/options.txt), pass an override
// through the launch builder. Relative resolves under game_dirs,
// absolute replaces:
//   instance.launch(&profile, JavaDistribution::Temurin)
//       .with_arguments()
//           .set(KEY_GAME_DIRECTORY, "runtime")      // → game_dirs/runtime
//           // or .set(KEY_GAME_DIRECTORY, "/mnt/games/custom")  // absolute
//           .done()
//       .run().await?;
```

### Check Directory Existence

```rust
use lighty_loaders::types::VersionInfo;

if instance.game_dir_exists() {
    println!("Game directory exists");
} else {
    println!("Game directory needs to be created");
}

if instance.java_dir_exists() {
    println!("Java directory exists");
}
```

### Get Both Directories

```rust
use lighty_loaders::types::VersionInfo;

let (game_dir, java_dir) = instance.paths();
println!("Game: {}", game_dir.display());
println!("Java: {}", java_dir.display());
```

## LightyVersionBuilder (Custom Server)

For custom servers using LightyUpdater:

```rust
use lighty_core::AppState;
use lighty_version::LightyVersionBuilder;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("MyLauncher")?;

    // Create LightyUpdater instance
    let modpack = LightyVersionBuilder::new(
        "my-modpack",
        "https://myserver.com/api",
    );

    println!("Instance: {}", modpack.name());
    println!("Server API: {}", modpack.loader_version());

    Ok(())
}
```

**Key Features**:
- Connects to custom server API
- Downloads modpack manifest from server
- Automatically handles mod updates
- See [LightyUpdater Repository](https://github.com/Lighty-Launcher/LightyUpdater) for server setup

## Using with LoaderExtensions

Both builders implement `VersionInfo`, enabling all loader operations:

```rust
use lighty_loaders::types::{VersionInfo, LoaderExtensions};

// VersionBuilder
let fabric = VersionBuilder::new(
    "fabric-1.21",
    Loader::Fabric,
    "0.16.9",
    "1.21.1",
);

// Get metadata (LoaderExtensions trait)
let metadata = fabric.get_metadata().await?;
println!("Main class: {}", metadata.main_class);
println!("Libraries: {}", metadata.libraries.len());

// Get specific parts
let libraries = fabric.get_libraries().await?;
let assets = fabric.get_assets().await?;

// LightyVersionBuilder
let modpack = LightyVersionBuilder::new(
    "modpack",
    "https://server.com/api",
);

// Same methods available
let metadata = modpack.get_metadata().await?;
```

## Using with InstanceControl

Both builders also support instance management:

```rust
use lighty_launch::InstanceControl;

// Create instance
let mut instance = VersionBuilder::new(
    "my-game",
    Loader::Fabric,
    "0.16.9",
    "1.21.1",
);

// Get metadata
let metadata = instance.get_metadata().await?;

// Calculate size
let size = instance.size_of_instance(&metadata);
println!("Total: {:.2} GB", size.total_gb());

// Get running PID
if let Some(pid) = instance.get_pid() {
    println!("Running with PID: {}", pid);

    // Close instance
    instance.close_instance(pid).await?;
}

// Delete instance
instance.delete_instance().await?;
```

## Complete Workflow

### Standard Loader Workflow

```rust
use lighty_core::AppState;
use lighty_version::VersionBuilder;
use lighty_loaders::types::{Loader, VersionInfo, LoaderExtensions};
use lighty_launch::InstanceControl;
use lighty_auth::{offline::OfflineAuth, Authenticator};
use lighty_java::JavaDistribution;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize AppState
    AppState::init("MyLauncher")?;

    // 2. Create instance
    let mut instance = VersionBuilder::new(
        "fabric-modded",
        Loader::Fabric,
        "0.16.9",
        "1.21.1",
    );

    println!("Instance: {}", instance.name());
    println!("Game dir: {}", instance.game_dirs().display());

    // 3. Get metadata
    let metadata = instance.get_metadata().await?;
    println!("Version: {}", metadata.id);
    println!("Libraries: {}", metadata.libraries.len());

    // 4. Calculate size
    let size = instance.size_of_instance(&metadata);
    println!("Size: {:.2} GB", size.total_gb());

    // 5. Authenticate
    let mut auth = OfflineAuth::new("Player123");

    #[cfg(not(feature = "events"))]
    let profile = auth.authenticate().await?;

    #[cfg(feature = "events")]
    let profile = auth.authenticate(None).await?;

    // 6. Launch
    instance.launch(&profile, JavaDistribution::Temurin)
        .run()
        .await?;

    println!("Game launched!");

    // 7. Get PID
    if let Some(pid) = instance.get_pid() {
        println!("Running with PID: {}", pid);
    }

    Ok(())
}
```

### Custom Server Workflow

```rust
use lighty_core::AppState;
use lighty_version::LightyVersionBuilder;
use lighty_loaders::types::{VersionInfo, LoaderExtensions};
use lighty_launch::InstanceControl;
use lighty_auth::{microsoft::MicrosoftAuth, Authenticator};
use lighty_java::JavaDistribution;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize AppState
    AppState::init("MyLauncher")?;

    // 2. Create LightyUpdater instance
    let mut modpack = LightyVersionBuilder::new(
        "survival-modpack",
        "https://play.myserver.com/api",
    );

    println!("Modpack: {}", modpack.name());
    println!("Server: {}", modpack.loader_version());

    // 3. Get metadata from server
    let metadata = modpack.get_metadata().await?;
    println!("Minecraft: {}", metadata.id);
    println!("Mods: {}", metadata.mods.as_ref().map_or(0, |m| m.len()));

    // 4. Authenticate
    let mut auth = MicrosoftAuth::new("your-client-id");
    auth.set_device_code_callback(|code, url| {
        println!("Visit: {}", url);
        println!("Code: {}", code);
    });

    #[cfg(not(feature = "events"))]
    let profile = auth.authenticate().await?;

    #[cfg(feature = "events")]
    let profile = auth.authenticate(None).await?;

    // 5. Launch modpack
    modpack.launch(&profile, JavaDistribution::Temurin)
        .run()
        .await?;

    println!("Modpack launched!");

    Ok(())
}
```

## Directory Structure

Both builders create the following structure:

```
game_dirs/
├── {instance-name}/         # Instance root
│   ├── versions/
│   │   └── {version}/
│   │       ├── {version}.json
│   │       └── {version}.jar
│   ├── libraries/           # Java libraries
│   ├── assets/              # Game assets
│   ├── mods/                # Mod files (if applicable)
│   ├── config/              # Mod configurations
│   └── saves/               # Worlds

java_dirs/
└── jre/                     # Java runtimes
    ├── java-8/
    ├── java-17/
    └── java-21/
```

## Exports

**In lighty_version**:
```rust
use lighty_version::{
    VersionBuilder,
    LightyVersionBuilder,
};
```

**In lighty_launcher**:
```rust
use lighty_launcher::{
    version::{VersionBuilder, LightyVersionBuilder},
    // or via prelude
    prelude::*,  // Includes VersionBuilder
};
```

## Related Documentation

- [Overview](./overview.md) - Architecture and design
- [Exports](./exports.md) - Complete export reference
- [VersionBuilder](./version-builder.md) - Standard builder details
- [LightyVersionBuilder](./lighty-version-builder.md) - Custom server builder details
- [lighty-loaders Traits](../../loaders/docs/traits.md) - VersionInfo and LoaderExtensions

## Related Crates

- **[lighty-loaders](../../loaders/README.md)** - VersionInfo trait and loaders
- **[lighty-core](../../core/README.md)** - AppState for launcher paths
- **[lighty-launch](../../launch/README.md)** - Uses VersionBuilder for launching
- **[lighty-auth](../../auth/README.md)** - User authentication
- **[lighty-java](../../java/README.md)** - Java runtime management
