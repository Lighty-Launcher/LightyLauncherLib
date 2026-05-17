# Forge Loader

Traditional mod loader with extensive mod support and long history.

## Overview

**Status**: Stable
**MC Versions**: 1.5.2+ (legacy 1.5.2 → 1.12.2 and modern 1.13+ both supported)
**Feature Flag**: `forge` (single flag — the dispatcher picks legacy vs modern from the installer schema)
**API**: MinecraftForge Maven

Forge is the original and most widely used Minecraft mod loader with the largest mod ecosystem.

## Current Status

Both legacy (1.5.2 → 1.12.2) and modern (≥ 1.13) installer schemas are
fully implemented and ship in the same `forge` feature flag — the
loader detects the right pipeline by reading the installer's
`install_profile.json`. Microsoft auth (`UserProfile.provider =
AuthProvider::Microsoft { .. }`) plus the launch placeholders
(`${auth_xuid}`, `${clientid}`, `${user_type} = "msa"`) all land in the
JVM args automatically through `build_arguments()`.

## Usage

### Modern Forge (≥ 1.13)

```rust
use lighty_launcher::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("MyLauncher")?;

    let mut auth = OfflineAuth::new("Player");
    let profile = auth.authenticate(None).await?;

    // VersionBuilder::new(name, Loader::Forge, loader_version, mc_version)
    let mut forge = VersionBuilder::new(
        "forge-1.21.8",
        Loader::Forge,
        "58.1.0",   // Forge version (without the MC prefix)
        "1.21.8",   // Minecraft version
    );

    forge
        .launch(&profile, JavaDistribution::Temurin)
        .run()
        .await?;
    Ok(())
}
```

### Legacy Forge (1.5.2 → 1.12.2)

Same `forge` feature flag, same API — only the loader version differs:

```rust
let mut forge = VersionBuilder::new(
    "forge-1.12.2",
    Loader::Forge,
    "14.23.5.2860",   // Legacy loader version
    "1.12.2",
);
```

The runtime dispatches to the legacy `forge_legacy` pipeline (no
processors, embedded `versionInfo`, universal JAR extracted from the
installer) when it sees the legacy `install_profile.json` schema.

## Exports

**In lighty_loaders**: `lighty_loaders::loaders::forge`
**In lighty_launcher**: `lighty_launcher::loaders::forge`

## API Endpoints

### Promotions (recommended / latest)
```
GET https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json
```

### Per-MC index
```
GET https://files.minecraftforge.net/net/minecraftforge/forge/index_{mc}.html
```

### Installer
```
GET https://maven.minecraftforge.net/net/minecraftforge/forge/{mc}-{loader}/forge-{mc}-{loader}-installer.jar
```

## Version Format

The maven coordinates for the installer are always `{mc}-{loader}`:

| Era | Example coordinate |
|-----|--------------------|
| Modern (≥ 1.13) | `1.21.8-58.1.0` |
| Legacy (1.5.2 → 1.12.2) | `1.12.2-14.23.5.2860` |

## Installation Process

### Modern (≥ 1.13)

1. Download installer JAR (cached under `<instance>/.forge/`)
2. Read `install_profile.json` and `version.json` directly from the JAR
3. Merge vanilla `version.json` + Forge `version.json` libraries
   (dedup by `group:artifact[:classifier]` — the classifier is required
   to keep `forge:universal` and `forge:client` side-by-side, both must
   be on the classpath or FML crashes with *"Failed to find system mod:
   forge"*)
4. Download all libraries in parallel (vanilla + Forge)
5. Run client-side processors from `install_profile.json` (typically
   `binarypatcher` for the patched client JAR; server-side processors
   are filtered out)
6. Build launch args from `Arguments` (game + jvm) with the
   `UserProfile`-derived placeholders

### Legacy (1.5.2 → 1.12.2)

1. Download installer ZIP (older versions use `.zip` instead of `.jar`)
2. Parse the embedded `install_profile.json` legacy schema
3. Extract the universal JAR to the libraries tree
4. No processors — the universal JAR ships ready to run

## Mod Ecosystem

Forge has the **largest mod ecosystem** in Minecraft:
- Thousands of mods available
- CurseForge/Modrinth support (see [`mods`](../../mods/) docs and the
  `examples/mods/` examples)
- Long history (2011+)

## Comparison with Other Loaders

| Feature | Forge | NeoForge | Fabric |
|---------|-------|----------|--------|
| Mod count | Highest | Growing | High |
| Performance | Good | Better | Best |
| MC support | 1.5.2+ | 1.20.1+ | 1.14+ |
| Complexity | High | Medium | Low |
| Load time | Slower | Medium | Fast |

## Related Documentation

- [NeoForge](./neoforge.md) — Modern Forge fork
- [Vanilla](./vanilla.md) — Base Minecraft
- [How to Use](../how-to-use.md) — General usage
- [`examples/forge.rs`](../../../../examples/forge.rs) — Runnable example
