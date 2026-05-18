# Forge

The original Minecraft mod loader. The `forge` feature flag covers
**both legacy (1.5.2 → 1.12.2) and modern (≥ 1.13) Forge** — the
loader detects which install schema to use by reading the installer's
`install_profile.json`. No separate `forge_legacy` feature exists.

| Field | Value |
|---|---|
| Status | stable |
| MC versions | 1.5.2 → latest |
| Feature flag | `forge` |
| Provider | MinecraftForge Maven |
| Module | `lighty_loaders::forge` |
| Repository singleton | `forge::FORGE` |

## Use it

### Modern (≥ 1.13)

```rust
use lighty_core::AppState;
use lighty_loaders::Loader;
use lighty_version::VersionBuilder;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("LightyLauncher")?;

    let v = VersionBuilder::new("forge-1.21.8", Loader::Forge, "58.1.0", "1.21.8");
    // hand `v` to lighty-launch — installer downloads + runs processors automatically
    let _ = v;
    Ok(())
}
```

### Legacy (1.5.2 → 1.12.2)

Same builder shape — only the loader version differs:

```rust
use lighty_loaders::Loader;
use lighty_version::VersionBuilder;

let v = VersionBuilder::new("forge-1.12.2", Loader::Forge, "14.23.5.2860", "1.12.2");
let _ = v;
```

`Query::extract` reads the installer's `install_profile.json` and
dispatches to the legacy pipeline (no processors, embedded
`versionInfo`, universal JAR extracted from the installer) when it
recognises the old schema.

## Microsoft auth + placeholders

Microsoft auth (`UserProfile.provider = AuthProvider::Microsoft { .. }`)
plus the launch placeholders `${auth_xuid}`, `${clientid}` and
`${user_type} = "msa"` are wired through `UserProfile` exactly like
on the other loaders — nothing Forge-specific to do.

## API endpoints

```
GET https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json
GET https://files.minecraftforge.net/net/minecraftforge/forge/index_{mc}.html
GET https://maven.minecraftforge.net/net/minecraftforge/forge/{mc}-{loader}/forge-{mc}-{loader}-installer.jar
```

Maven coordinates are always `{mc}-{loader}` (e.g. `1.21.8-58.1.0`,
`1.12.2-14.23.5.2860`).

## Install pipeline

Modern:

1. Download the installer JAR (cached under `<instance>/.forge/`).
2. Read `install_profile.json` + `version.json` from the JAR.
3. Merge vanilla + Forge libraries, deduplicating by
   `group:artifact[:classifier]` (the `:classifier` part is required —
   `forge:universal` and `forge:client` must both stay on the
   classpath).
4. Download all libraries in parallel.
5. Run the client-side processors from `install_profile.json`
   (typically `binarypatcher`). Server-side processors are filtered
   out.
6. Build launch args with `UserProfile`-derived placeholders.

Legacy:

1. Download the installer (ZIP for the oldest, JAR otherwise).
2. Parse the legacy `install_profile.json` schema.
3. Extract the universal JAR to the libraries tree.
4. No processors — the universal JAR ships ready to run.

Step-by-step launch-side detail in
[`../../../launch/docs/installation.md`](../../../launch/docs/installation.md).

## Public helpers

```rust
pub use lighty_loaders::forge::forge::{
    FORGE, ForgeQuery, ForgeRawData,
    extract_install_profile_libraries_modern,
    build_installer_url,
    installer_cache_path,
};
pub use lighty_loaders::forge::forge_legacy::{
    is_legacy_forge,
    legacy_installer_path,
    InstallProfileKind,
};
```

`is_legacy_forge(mc: &str) -> bool` is the runtime branch the
dispatcher uses to pick the legacy vs modern path.

## Events

Standard set — see [`../events.md`](../events.md).

## Mod ecosystem

Largest Minecraft mod ecosystem (2011+). Auto-pull mods through
Modrinth / CurseForge — see
[`../../../modsloader/docs/mods.md`](../../../modsloader/docs/mods.md).

## See also

- [`vanilla.md`](./vanilla.md) — base manifest source
- [`neoforge.md`](./neoforge.md) — modern Forge fork
- [`../../../launch/docs/installation.md`](../../../launch/docs/installation.md)
  — installer pipeline detail (processors, classifier dedup)
- [`../../../modsloader/docs/mods.md`](../../../modsloader/docs/mods.md)
  — auto-pull mods
- Runnable example: `examples/forge.rs`
