# Quilt

Fabric fork with extra niceties (lossless extension API, broader mod
compatibility). Same merge model as Fabric — pulls the Vanilla
manifest, overlays the Quilt loader profile.

| Field | Value |
|---|---|
| Status | stable |
| MC versions | 1.14+ |
| Feature flag | `quilt` |
| Provider | QuiltMC meta API |
| Module | `lighty_loaders::quilt` |
| Repository singleton | `quilt::QUILT` |

## Use it

```rust,no_run
use lighty_core::AppState;
use lighty_loaders::{Loader, LoaderExtensions};
use lighty_version::VersionBuilder;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("LightyLauncher")?;
    let v = VersionBuilder::new("quilt-1.21", Loader::Quilt, "0.27.1", "1.21.1");
    let meta = v.get_metadata().await?;
    println!("libraries: {}", meta.libraries.len());
    Ok(())
}
```

## Queries

```rust,ignore
pub enum QuiltQuery { QuiltBuilder, Libraries }
```

## API endpoints

```
GET https://meta.quiltmc.org/v3/versions/loader
GET https://meta.quiltmc.org/v3/versions/game
GET https://meta.quiltmc.org/v3/versions/loader/{mc}/{loader}/profile/json
```

## Compatibility

Quilt loads most Fabric mods unmodified. Quilt-specific mods only run
on Quilt. Use the Quilted Fabric API for the Fabric API surface.

## Events

Same set as Fabric — `FetchingData / DataFetched / ManifestCached /
ManifestNotFound` + `MergingLoaderData / DataMerged` with
`overlay_loader: "Quilt"`. See [`../events.md`](../events.md).

## See also

- [`fabric.md`](./fabric.md) — same merge model
- [`vanilla.md`](./vanilla.md) — base manifest source
- [`../../../modsloader/docs/mods.md`](../../../modsloader/docs/mods.md)
  — auto-pull mods
