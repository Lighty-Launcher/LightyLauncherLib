# Vanilla

Pure Mojang Minecraft — no mod loader on top. Backs every other
loader (Fabric / Quilt / Forge / NeoForge all merge with the Vanilla
manifest).

| Field | Value |
|---|---|
| Status | stable |
| MC versions | all |
| Feature flag | `vanilla` |
| Provider | Mojang piston-meta API |
| Module | `lighty_loaders::vanilla` |
| Repository singleton | `vanilla::VANILLA` |

## Use it

```rust,no_run
use lighty_core::AppState;
use lighty_loaders::{Loader, LoaderExtensions};
use lighty_version::VersionBuilder;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("LightyLauncher")?;

    let v = VersionBuilder::new("vanilla-1.21", Loader::Vanilla, "", "1.21.1");
    let meta = v.get_metadata().await?;

    println!("main_class: {}", meta.main_class);
    println!("libraries:  {}", meta.libraries.len());
    Ok(())
}
```

The loader version is empty for Vanilla — only `minecraft_version`
matters.

## Queries

```rust,ignore
pub enum VanillaQuery {
    VanillaBuilder,   // full metadata
    Libraries,        // just libraries
    MainClass,
    Natives,
    JavaVersion,
    Assets,
}
```

Use the matching `LoaderExtensions::get_*` method instead of poking
the `VanillaQuery` enum directly — same caching, narrower API.

## API endpoints

```
GET https://piston-meta.mojang.com/mc/game/version_manifest_v2.json
GET https://piston-meta.mojang.com/v1/packages/{sha1}/{version}.json
```

The manifest is fetched once per process and reused for every
`(version)` lookup; the per-version JSON is cached separately.

## Events

Standard set — see [`../events.md`](../events.md):
`FetchingData → DataFetched` on first fetch, `ManifestCached` on a
warm hit, `ManifestNotFound` if the version is missing from the
manifest.

## See also

- [`fabric.md`](./fabric.md), [`quilt.md`](./quilt.md),
  [`forge.md`](./forge.md), [`neoforge.md`](./neoforge.md) — loaders
  that merge with this one
- [`../traits.md`](../traits.md) — `LoaderExtensions::get_*`
- [`../cache.md`](../cache.md) — TTL behaviour
