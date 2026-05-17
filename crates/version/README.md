# lighty-version

Version management and builders for [LightyLauncher](https://crates.io/crates/lighty-launcher).

## Overview

**Version**: 26.5.1
**Part of**: [LightyLauncher](https://crates.io/crates/lighty-launcher)

`lighty-version` provides version builders that implement the `VersionInfo` trait from `lighty-loaders`, enabling version management for Minecraft instances.

## Features

- **VersionBuilder** - Standard version builder for all loaders
- **LightyVersionBuilder** - Custom server version builder for LightyUpdater
- **VersionInfo Implementation** - Implements core version trait
- **Directory Management** - Configurable game and Java directories
- **Type Safety** - Strongly typed version information

## Quick Start

```toml
[dependencies]
lighty-version = "26.5.1"
```

### VersionBuilder (Standard Loaders)

```rust
use lighty_core::AppState;
use lighty_version::VersionBuilder;
use lighty_loaders::types::Loader;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("MyLauncher")?;

    // Create instance with Fabric loader
    let instance = VersionBuilder::new(
        "my-instance",         // Instance name
        Loader::Fabric,        // Loader type
        "0.16.9",             // Loader version
        "1.21.1",             // Minecraft version
    );

    println!("Instance: {}", instance.name());
    println!("Game dir: {}", instance.game_dirs().display());

    Ok(())
}
```

### LightyVersionBuilder (Custom Server)

```rust
use lighty_core::AppState;
use lighty_version::LightyVersionBuilder;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("MyLauncher")?;

    // Create instance for custom server with LightyUpdater
    let instance = LightyVersionBuilder::new(
        "my-modpack",                  // Instance name
        "https://myserver.com/api",    // Server API URL
    );

    println!("Server: {}", instance.loader_version());

    Ok(())
}
```

### With Custom Directories

```rust
use std::path::PathBuf;

// Custom Java directory
let instance = VersionBuilder::new(
    "custom",
    Loader::Vanilla,
    "",
    "1.21.1",
)
.with_custom_java_dir(PathBuf::from("/usr/lib/jvm/java-21"));

// To relocate the game runtime (mods/saves/options.txt), use the
// launch-side knob:
//   .launch(...).with_arguments().set(KEY_GAME_DIRECTORY, "/opt/...").done()
```

## Core Types

| Type | Description |
|------|-------------|
| **VersionBuilder** | Standard version builder for all loaders |
| **LightyVersionBuilder** | Custom server builder for LightyUpdater |

Both implement `VersionInfo` from `lighty-loaders`.

## Documentation

📚 **[Complete Documentation](./docs)**

| Guide | Description |
|-------|-------------|
| [How to Use](./docs/how-to-use.md) | Practical usage guide with examples |
| [Overview](./docs/overview.md) | Architecture and design |
| [Exports](./docs/exports.md) | Complete export reference |
| [VersionBuilder](./docs/version-builder.md) | Standard version builder details |
| [LightyVersionBuilder](./docs/lighty-version-builder.md) | Custom server builder details |

## Related Crates

- **[lighty-launcher](../../../README.md)** - Main package
- **[lighty-loaders](../loaders/README.md)** - VersionInfo trait and loaders
- **[lighty-core](../core/README.md)** - AppState for launcher paths
- **[lighty-launch](../launch/README.md)** - Uses VersionBuilder for launching

## License

MIT
