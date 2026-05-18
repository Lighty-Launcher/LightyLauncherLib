# NeoForge

Modern fork of Forge focused on newer Minecraft versions. Cleaner
internals, faster-moving development. Two Maven artifact eras —
both supported transparently.

| Field | Value |
|---|---|
| Status | stable |
| MC versions | 1.20.1 (old artifact path) / 1.20.2+ (modern path) |
| Feature flag | `neoforge` |
| Provider | NeoForged Maven |
| Module | `lighty_loaders::neoforge` |
| Repository singleton | `neoforge::NEOFORGE` |

## Use it

```rust
use lighty_core::AppState;
use lighty_loaders::Loader;
use lighty_version::VersionBuilder;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("LightyLauncher")?;

    let v = VersionBuilder::new("neoforge-1.21.8", Loader::NeoForge, "21.8.53", "1.21.8");
    let _ = v; // hand to lighty-launch
    Ok(())
}
```

## Two artifact paths

| MC version | Maven coordinates |
|---|---|
| ≤ 1.20.1 | `net.neoforged:forge:{mc}-{loader}` |
| ≥ 1.20.2 | `net.neoforged:neoforge:{loader}` |

The loader detects which path to use from the Minecraft version. The
modern pipeline writes
`{LIBRARY_DIR}/net/neoforged/neoforge/.../neoforge-{v}-client.jar`
from the embedded binary patches.

NeoForge version numbers track the Minecraft minor:

| Minecraft | NeoForge major |
|---|---|
| 1.21.x | 21.x.y |
| 1.20.6 | 20.6.x |
| 1.20.4 | 20.4.x |
| 1.20.2 | 20.2.x |
| 1.20.1 | (separate `net.neoforged:forge` path) |

## API endpoints

```
# Version listing
GET https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/neoforge

# Installer (modern, ≥ 1.20.2)
GET https://maven.neoforged.net/releases/net/neoforged/neoforge/{loader}/neoforge-{loader}-installer.jar

# Installer (old, = 1.20.1)
GET https://maven.neoforged.net/releases/net/neoforged/forge/{mc}-{loader}/forge-{mc}-{loader}-installer.jar
```

## Install pipeline

1. Download the installer JAR (cached under `<instance>/.forge/` —
   shared with Forge by design, same on-disk layout).
2. Read `install_profile.json` + `version.json` from the JAR.
3. Merge vanilla + NeoForge libraries, deduplicating by
   `group:artifact[:classifier]` for classifier safety (NeoForge's
   `version.json` doesn't currently use the `:universal`/`:client`
   split that Forge does, but the dedup logic is symmetric).
4. Download all libraries in parallel.
5. Run the client-side processors (binary patch, jar splits, …).
6. Build launch args with `UserProfile`-derived placeholders
   (`${auth_xuid}`, `${clientid}`, `${user_type} = "msa"` for
   Microsoft auth — wired identically to Forge).

Step-by-step launch-side detail in
[`../../../launch/docs/installation.md`](../../../launch/docs/installation.md).

## Mod ecosystem

- Recent Forge mods (1.20.2+) are usually source-compatible.
- NeoForge-specific mods are growing fast.
- CurseForge / Modrinth integration via the `modrinth` /
  `curseforge` feature flags — see
  [`../../../modsloader/docs/mods.md`](../../../modsloader/docs/mods.md).

## Events

Standard set — see [`../events.md`](../events.md).

## See also

- [`forge.md`](./forge.md) — traditional Forge loader
- [`vanilla.md`](./vanilla.md) — base manifest
- [`../../../launch/docs/installation.md`](../../../launch/docs/installation.md)
  — installer pipeline detail
- Runnable example: `examples/neoforge.rs`
