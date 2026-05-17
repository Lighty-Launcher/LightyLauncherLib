# OptiFine

Performance and graphics optimization mod (not a true mod loader).

## Overview

**Status**: Experimental
**MC Versions**: Most versions
**Feature Flag**: Requires `vanilla` feature
**Type**: Mod, not a loader

OptiFine is a graphics optimization mod, not a mod loader. It's included in the loader system for convenience but behaves differently.

## Important Note

OptiFine is **not a mod loader** like Fabric or Forge. It's a standalone mod that:
- Improves graphics performance
- Adds visual enhancements
- Can be installed with Forge or as standalone
- Has limited mod compatibility

## Usage

OptiFine is typically installed:

### As Standalone with Vanilla

Place OptiFine JAR in the `mods/` directory of a Vanilla instance.

### With Fabric (via OptiFabric)

Use OptiFabric mod to run OptiFine on Fabric:

1. Install Fabric
2. Install OptiFabric mod
3. Install OptiFine

```rust
use lighty_launcher::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("MyLauncher")?;

    // Create Fabric instance
    let instance = VersionBuilder::new(
        "fabric-optifine",
        Loader::Fabric,
        "0.16.9",
        "1.21.1",
    );

    // Manually install OptiFine JAR to mods/ directory
    // (not handled by loader system)

    Ok(())
}
```

## Exports

**In lighty_loaders**: `lighty_loaders::loaders::optifine`
**In lighty_launcher**: `lighty_launcher::loaders::optifine`

**Note**: OptiFine module exists but is experimental and not recommended for production use.

## Version Format

OptiFine versions follow this pattern:
```
HD_{type}_{version}
```

Examples:
- `HD_U_I9` (1.21.x)
- `HD_U_I8` (1.21.x)
- `HD_U_H9` (1.20.4)

Where:
- `HD` = High Definition
- `U` = Ultra
- Letter = Sub-version (H, I, J...)
- Number = Patch version

## Features

OptiFine provides:
- **Performance**: Better FPS
- **Graphics**: Shaders, custom textures
- **Optimization**: Fog, particles, animations
- **Quality**: HD textures, better lighting

## Installation Methods

### Method 1: Direct JAR (Vanilla)

1. Download OptiFine JAR
2. Place in `mods/` directory
3. Launch with Vanilla

### Method 2: With Fabric (OptiFabric)

1. Install Fabric
2. Download OptiFabric from CurseForge/Modrinth
3. Download OptiFine
4. Place both in `mods/` directory

### Method 3: With Forge

OptiFine can be installed directly as a Forge mod.

## Compatibility

### Works With
- Vanilla Minecraft
- Forge (most versions)
- Fabric (via OptiFabric)

### Limited Compatibility
- Some performance mods (Sodium, etc.)
- Some shader mods
- Heavily modded instances

## Why Experimental?

OptiFine support in lighty-loaders is experimental because:
- Not a true loader
- No official API for metadata
- Installation is manual (mod file)
- Version detection is complex
- Compatibility issues

## Recommended Approach

Instead of using OptiFine as a "loader", use it as a mod:

```rust
use lighty_launcher::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("MyLauncher")?;

    // 1. Create base instance (Vanilla or Fabric)
    let instance = VersionBuilder::new(
        "my-instance",
        Loader::Fabric,  // or Loader::Vanilla
        "0.16.9",        // or ""
        "1.21.1",
    );

    // 2. Manually download OptiFine JAR
    let mods_dir = AppState::data_dir()
        .join("instances")
        .join("my-instance")
        .join("mods");

    use lighty_launcher::macros::mkdir;
    mkdir!(&mods_dir);

    // 3. Download OptiFine to mods_dir
    // (using lighty-core download utilities)

    // 4. Launch normally
    let metadata = instance.get_metadata().await?;

    Ok(())
}
```

## Alternatives

For better performance without compatibility issues, consider:
- **Sodium** (Fabric) - Better performance than OptiFine
- **Iris** (Fabric) - Shader support
- **Lithium** (Fabric) - Server-side optimization
- **Phosphor** (Fabric) - Lighting optimization

## Related Documentation

- [Vanilla](./vanilla.md) - Base Minecraft
- [Fabric](./fabric.md) - Fabric loader (use with OptiFabric)
- [How to Use](../how-to-use.md) - General usage
