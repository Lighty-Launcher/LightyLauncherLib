# lighty-loaders

Mod loader support for Minecraft: Vanilla, Fabric, Quilt, Forge, NeoForge, OptiFine, and custom loaders.

## Overview

**Version**: 26.5.1
**Part of**: [LightyLauncher](https://crates.io/crates/lighty-launcher)

Provides a unified trait-based API for managing different Minecraft mod loaders with smart caching and metadata resolution.

## Quick Start

```rust
use lighty_launcher::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize AppState
    AppState::init("MyLauncher")?;

    // Create instance with Fabric loader
    let mut instance = VersionBuilder::new(
        "my-instance",
        Loader::Fabric,
        "0.16.9",      // Fabric loader version
        "1.21.1",      // Minecraft version
    );

    // Get metadata (automatically fetched and cached)
    let metadata = instance.get_metadata().await?;

    trace_info!("Loaded {} with {} libraries", metadata.id, metadata.libraries.len());

    Ok(())
}
```

## Supported Loaders

| Loader | Feature Flag | Status | MC Versions |
|--------|-------------|--------|-------------|
| Vanilla | `vanilla` | Stable | All |
| Fabric | `fabric` | Stable | 1.14+ |
| Quilt | `quilt` | Stable | 1.14+ |
| NeoForge | `neoforge` | Stable | 1.20.2+ |
| Forge | `forge` | In Progress | 1.13+ |
| Forge Legacy | `forge_legacy` | In Progress | 1.7-1.12 |
| LightyUpdater | `lighty_updater` | Stable | Custom |
| OptiFine | (vanilla feature) | Experimental | Most |

## Features

- **Trait-based system**: `VersionInfo` and `LoaderExtensions` for extensibility
- **Smart caching**: Dual-layer cache (raw + query) with configurable TTL
- **Query system**: Flexible metadata queries (full, libraries only, etc.)
- **Event integration**: Progress tracking via `lighty-event`
- **Feature flags**: Compile only what you need

## Installation

```toml
[dependencies]
# With all loaders
lighty-loaders = { version = "26.5.1", features = ["all-loaders"] }

# With specific loaders
lighty-loaders = { version = "26.5.1", features = ["vanilla", "fabric", "quilt"] }
```

## Core Traits

### VersionInfo

Defines version information for any instance:

```rust
use lighty_launcher::loaders::VersionInfo;

// Already implemented by VersionBuilder
let instance = VersionBuilder::new("name", Loader::Vanilla, "", "1.21.1");

println!("Name: {}", instance.name());
println!("Minecraft: {}", instance.minecraft_version());
println!("Game dir: {}", instance.game_dirs().display());
```

### LoaderExtensions

Extension methods for fetching loader metadata:

```rust
use lighty_launcher::loaders::LoaderExtensions;

// Get full metadata
let metadata = instance.get_metadata().await?;

// Or get specific parts
let libraries = instance.get_libraries().await?;
let assets = instance.get_assets().await?;
```

## Exports

### In `lighty_loaders`

```rust
use lighty_loaders::{
    // Types
    types::{Loader, VersionInfo, LoaderExtensions, InstanceSize},

    // Loaders modules (feature-gated)
    loaders::{vanilla, fabric, quilt, neoforge, forge, lighty_updater, optifine},

    // Utils
    utils::{cache, error, manifest, query},
};
```

### In `lighty_launcher` (re-exports)

```rust
use lighty_launcher::prelude::*;
// or
use lighty_launcher::loaders::{Loader, VersionInfo, LoaderExtensions, InstanceSize};
```

## Documentation

📚 **[Complete Documentation](./docs)**

| Guide | Description |
|-------|-------------|
| [How to Use](./docs/how-to-use.md) | Simple guide with practical examples |
| [Overview](./docs/overview.md) | Architecture and design |
| [Traits](./docs/traits.md) | VersionInfo and LoaderExtensions explained |
| [Query System](./docs/query.md) | How queries work |
| [Cache System](./docs/cache.md) | Caching architecture |
| [Events](./docs/events.md) | LoaderEvent types |
| [Exports](./docs/exports.md) | All exports and re-exports |
| **Loaders** | |
| [Vanilla](./docs/loaders/vanilla.md) | Pure Minecraft |
| [Fabric](./docs/loaders/fabric.md) | Lightweight mod loader |
| [Quilt](./docs/loaders/quilt.md) | Fabric fork |
| [NeoForge](./docs/loaders/neoforge.md) | Modern Forge fork |
| [Forge](./docs/loaders/forge.md) | Traditional mod loader |
| [LightyUpdater](./docs/loaders/lighty_updater.md) | Custom loader system |
| [OptiFine](./docs/loaders/optifine.md) | Graphics optimization |

## Related Crates

- **[lighty-launcher](../../../README.md)** - Main package
- **[lighty-version](../version/README.md)** - VersionBuilder implementation
- **[lighty-launch](../launch/README.md)** - Launch system
- **[lighty-event](../event/README.md)** - Event system

## License

MIT
