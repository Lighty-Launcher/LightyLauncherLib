# Quilt Loader

Fork of Fabric with additional features and improvements.

## Overview

**Status**: Stable
**MC Versions**: 1.14+
**Feature Flag**: `quilt`
**API**: QuiltMC official API

Quilt is a Fabric fork that aims to provide better mod compatibility and additional features while maintaining Fabric mod support.

## Usage

```rust
use lighty_launcher::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("MyLauncher")?;

    let instance = VersionBuilder::new(
        "quilt-1.21",
        Loader::Quilt,
        "0.27.1",      // Quilt loader version
        "1.21.1",      // Minecraft version
    );

    let metadata = instance.get_metadata().await?;

    println!("Quilt {} with {} libraries", metadata.id, metadata.libraries.len());

    Ok(())
}
```

## Exports

**In lighty_loaders**: `lighty_loaders::loaders::quilt`  
**In lighty_launcher**: `lighty_launcher::loaders::quilt`

## API Endpoints

### Loader Versions
```
GET https://meta.quiltmc.org/v3/versions/loader
```

### Game Versions
```
GET https://meta.quiltmc.org/v3/versions/game
```

### Loader Profile
```
GET https://meta.quiltmc.org/v3/versions/loader/{minecraft}/{loader}/profile/json
```

## Query Types

- `QuiltBuilder` - Full metadata (Quilt + Vanilla merged)
- `Libraries` - Only libraries

## Merging with Vanilla

Same process as Fabric:
1. Fetch Vanilla metadata
2. Fetch Quilt loader profile
3. Merge libraries
4. Merge arguments
5. Return combined metadata

## Comparison with Fabric

| Feature | Fabric | Quilt |
|---------|--------|-------|
| Mod compatibility | Fabric mods only | Fabric + Quilt mods |
| API stability | Stable | Stable |
| Development | Community | Fork + improvements |
| Performance | Excellent | Excellent |

## Mod Support

Quilt can run:
- Quilt-specific mods
- Most Fabric mods (high compatibility)

## Related Documentation

- [Fabric](./fabric.md) - Very similar architecture
- [Vanilla](./vanilla.md) - Base Minecraft
