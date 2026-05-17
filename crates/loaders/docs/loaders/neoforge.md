# NeoForge Loader

Modern fork of Forge focused on newer Minecraft versions.

## Overview

**Status**: Stable
**MC Versions**: 1.20.1 (old `forge` artifact path) and 1.20.2+ (modern `neoforge` artifact path)
**Feature Flag**: `neoforge`
**API**: NeoForged Maven

NeoForge is a modern rewrite and fork of MinecraftForge, focused on
newer Minecraft versions with cleaner internals and faster moving
development.

## Current Status

Both artifact path eras are supported:

- **MC ≤ 1.20.1** → coordinates `net.neoforged:forge:{mc}-{loader}`
- **MC ≥ 1.20.2** → coordinates `net.neoforged:neoforge:{loader}`

The loader detects which path to use from the Minecraft version and
runs the installer's processor pipeline (the modern pipeline writes
`{LIBRARY_DIR}/net/neoforged/neoforge/.../neoforge-{v}-client.jar` from
the embedded binary patches). Microsoft auth + the launch placeholders
(`${auth_xuid}`, `${clientid}`, `${user_type} = "msa"`) are wired
through `UserProfile` exactly like the Forge path.

## Usage

```rust
use lighty_launcher::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("MyLauncher")?;

    let mut auth = OfflineAuth::new("Player");
    let profile = auth.authenticate(None).await?;

    // VersionBuilder::new(name, Loader::NeoForge, loader_version, mc_version)
    let mut neoforge = VersionBuilder::new(
        "neoforge-1.21.8",
        Loader::NeoForge,
        "21.8.53",   // NeoForge version
        "1.21.8",    // Minecraft version
    );

    neoforge
        .launch(&profile, JavaDistribution::Temurin)
        .run()
        .await?;
    Ok(())
}
```

## Exports

**In lighty_loaders**: `lighty_loaders::loaders::neoforge`
**In lighty_launcher**: `lighty_launcher::loaders::neoforge`

## API Endpoints

### Version listing
```
GET https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/neoforge
```

### Installer (modern, ≥ 1.20.2)
```
GET https://maven.neoforged.net/releases/net/neoforged/neoforge/{loader}/neoforge-{loader}-installer.jar
```

### Installer (old, = 1.20.1)
```
GET https://maven.neoforged.net/releases/net/neoforged/forge/{mc}-{loader}/forge-{mc}-{loader}-installer.jar
```

## Version Format

NeoForge versions follow `{major}.{minor}.{patch}`, where the major
maps to the Minecraft minor:

| Minecraft | NeoForge major |
|-----------|---------------|
| 1.21.x | 21.x.y |
| 1.20.6 | 20.6.x |
| 1.20.4 | 20.4.x |
| 1.20.2 | 20.2.x |
| 1.20.1 | (`net.neoforged:forge`, separate path) |

Examples:
- `21.8.53` for MC 1.21.8
- `20.4.109` for MC 1.20.4

## Installation Process

1. Download NeoForge installer JAR (cached under `<instance>/.forge/`,
   shared with the Forge cache by design — same on-disk layout)
2. Read `install_profile.json` + `version.json` from the JAR
3. Merge vanilla + NeoForge libraries
   (dedup by `group:artifact[:classifier]` for classifier safety, even
   though NeoForge's `version.json` doesn't currently use the
   `:universal`/`:client` split that Forge does)
4. Download all libraries in parallel
5. Run the client-side processors (binary patch, jar splits, etc.)
6. Build launch args with the `UserProfile`-derived placeholders

## Mod Support

- Recent Forge mods (1.20.2+) — usually source-compatible
- NeoForge-specific mods
- CurseForge/Modrinth integration via the `mods` feature flags

## Comparison with Forge

| Feature | Forge | NeoForge |
|---------|-------|----------|
| MC versions | 1.5.2+ | 1.20.1+ |
| Codebase | Legacy | Modern rewrite |
| Mod compat | Extensive (old) | Modern mods |
| Development | Slower | Active |

## Related Documentation

- [Forge](./forge.md) — Traditional Forge loader
- [Vanilla](./vanilla.md) — Base Minecraft
- [How to Use](../how-to-use.md) — General usage
- [`examples/neoforge.rs`](../../../../examples/neoforge.rs) — Runnable example
