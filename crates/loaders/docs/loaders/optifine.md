# OptiFine

Graphics / performance mod — **not a real mod loader**. The module
exists for convenience and dispatch but installation is largely manual
and only marginally supported.

| Field | Value |
|---|---|
| Status | experimental |
| MC versions | most |
| Feature flag | always available (depends on `vanilla`) |
| Module | `lighty_loaders::optifine` |

## What it actually is

OptiFine ships as a standalone mod JAR. Three install modes exist in
the wild:

1. **Vanilla + OptiFine JAR in `mods/`** — works out of the box from
   the launcher's perspective; you provide the JAR.
2. **Forge + OptiFine** — OptiFine ships as a Forge mod for these
   versions. Use `Loader::Forge` and drop the JAR in `mods/`.
3. **Fabric + OptiFabric + OptiFine** — use `Loader::Fabric`, drop
   both JARs in `mods/`. OptiFabric is on
   [Modrinth](https://modrinth.com/mod/optifabric) and
   [CurseForge](https://www.curseforge.com/minecraft/mc-mods/optifabric).

In all three modes the loader you pass to `VersionBuilder::new` is
the *real* loader (Vanilla / Forge / Fabric). `Loader::Optifine` is
reserved for future first-class support and currently routes through
the Vanilla pipeline.

## Skeleton

```rust
use lighty_core::AppState;
use lighty_loaders::Loader;
use lighty_version::VersionBuilder;
# fn run() -> anyhow::Result<()> {
AppState::init("LightyLauncher")?;

// Real install: Fabric base + OptiFabric + OptiFine in mods/
let v = VersionBuilder::new("of-1.21", Loader::Fabric, "0.16.9", "1.21.1");
let _ = v;
# Ok(()) }
```

## Alternatives worth considering

For most modern setups, the Fabric performance stack outperforms
OptiFine and avoids the compatibility pain:

- **Sodium** — rendering rewrite
- **Iris** — shader pipeline (replaces OptiFine shaders)
- **Lithium** — server-side / tick optimisations
- **Indium** — Sodium-friendly Fabric Rendering API impl

All three are on Modrinth — `with_modrinth_mods(["sodium", "iris",
"lithium", "indium"], None)` via
[`../../../modsloader/docs/mods.md`](../../../modsloader/docs/mods.md).

## See also

- [`vanilla.md`](./vanilla.md), [`fabric.md`](./fabric.md),
  [`forge.md`](./forge.md) — the loaders OptiFine actually runs on
- [`../../../modsloader/docs/mods.md`](../../../modsloader/docs/mods.md)
  — pull mods automatically
