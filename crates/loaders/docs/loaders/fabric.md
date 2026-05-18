# Fabric

Lightweight mod loader with a big ecosystem. Merges its loader
profile with Vanilla — the launcher fetches both manifests, merges
libraries, overrides the main class.

| Field | Value |
|---|---|
| Status | stable |
| MC versions | 1.14+ official |
| Feature flag | `fabric` |
| Provider | FabricMC meta API |
| Module | `lighty_loaders::fabric` |
| Repository singleton | `fabric::FABRIC` |

## Use it

```rust,no_run
use lighty_core::AppState;
use lighty_loaders::{Loader, LoaderExtensions};
use lighty_version::VersionBuilder;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("LightyLauncher")?;

    let v = VersionBuilder::new("fabric-1.21", Loader::Fabric, "0.16.9", "1.21.1");
    let meta = v.get_metadata().await?;

    let fabric_libs = meta.libraries.iter()
        .filter(|l| l.name.starts_with("net.fabricmc:"))
        .count();
    println!("vanilla libs: {}", meta.libraries.len() - fabric_libs);
    println!("fabric libs:  {}", fabric_libs);
    Ok(())
}
```

## Queries

```rust,ignore
pub enum FabricQuery { FabricBuilder, Libraries }
```

`FabricBuilder` returns the merged Vanilla+Fabric metadata.

## API endpoints

```
GET https://meta.fabricmc.net/v2/versions/loader                              # list
GET https://meta.fabricmc.net/v2/versions/loader/{mc}/{loader}/profile/json   # profile
```

## Merge flow

1. Pull the Vanilla manifest for the same MC version (via
   `vanilla::VANILLA`).
2. Pull the Fabric loader profile.
3. Append Fabric libraries to the Vanilla library list.
4. Override `main_class` with the Fabric value if the profile
   provides one.
5. Merge JVM / game arguments.
6. Return the combined metadata.

Vanilla and Fabric have their own raw caches, so a TTL refresh on one
side doesn't invalidate the other. The merged result is cached under
the Fabric query key.

## Events

Standard `FetchingData / DataFetched / ManifestCached / ManifestNotFound`
plus `MergingLoaderData { base_loader: "Vanilla", overlay_loader:
"Fabric" }` / `DataMerged { … }` around the merge step. Full sequence
samples in [`../events.md`](../events.md).

## Mods

Fabric mods land in `<instance>/mods/`. To pull them automatically
from Modrinth / CurseForge, see
[`../../../modsloader/docs/mods.md`](../../../modsloader/docs/mods.md).

Fabric API (the framework most mods depend on) is itself a mod —
install it through Modrinth (`fabric-api`) or CurseForge.

## See also

- [`vanilla.md`](./vanilla.md) — base manifest source
- [`quilt.md`](./quilt.md) — sibling fork with same merge model
- [`../../../modsloader/docs/mods.md`](../../../modsloader/docs/mods.md)
  — auto-pull mods
- [`../traits.md`](../traits.md), [`../cache.md`](../cache.md)
