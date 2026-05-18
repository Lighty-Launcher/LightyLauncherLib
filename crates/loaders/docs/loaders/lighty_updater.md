# LightyUpdater

Custom-server loader for modpack-style distribution. The launcher
fetches its `(libraries, args, base-loader)` payload from a URL you
control, then merges it on top of a real base loader (Vanilla /
Fabric / Quilt / NeoForge / Forge).

| Field | Value |
|---|---|
| Status | stable |
| MC versions | server-defined |
| Feature flag | `lighty_updater` (pulls in `fabric`, `quilt`, `neoforge`, `forge` automatically) |
| Provider | your server |
| Module | `lighty_loaders::lighty_updater` |
| Builder | `lighty_version::LightyVersionBuilder` |

## Use it

```rust
use lighty_core::AppState;
use lighty_loaders::LoaderExtensions;
use lighty_version::LightyVersionBuilder;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("LightyLauncher")?;

    let v = LightyVersionBuilder::new(
        "my-modpack",                  // instance name
        "https://myserver.com/api",    // base URL of your endpoint
    );

    let meta = v.get_metadata().await?;
    println!("{} libs", meta.libraries.len());
    Ok(())
}
```

`LightyVersionBuilder` is documented in
[`../../../version/docs/how-to-use.md`](../../../version/docs/how-to-use.md).

## Server contract

```
GET {server_url}/metadata?mc_version={minecraft_version}
```

Response — `VersionMetaData`-compatible JSON, e.g.:

```json
{
  "id": "1.21.1",
  "type": "release",
  "mainClass": "net.minecraft.client.main.Main",
  "libraries": [
    {
      "name": "com.example:my-mod:1.0.0",
      "url":  "https://myserver.com/mods/my-mod-1.0.0.jar",
      "sha1": "abc...",
      "size": 1234567
    }
  ],
  "arguments": { "jvm": ["-Xmx4G"], "game": ["--username", "${auth_player_name}"] },
  "assetIndex": { "id": "16", "url": "https://piston-data.mojang.com/...", "totalSize": 500000000 }
}
```

## Picking a base loader

The server's payload includes a `loader` string in its `ServerInfo`
block. The launcher resolves it to a real `Loader` variant:

| `loader` value | Resolved to |
|---|---|
| `"vanilla"` | `Loader::Vanilla` |
| `"fabric"` | `Loader::Fabric` |
| `"quilt"` | `Loader::Quilt` |
| `"neoforge"` | `Loader::NeoForge` |
| `"forge"` | `Loader::Forge` |

Unknown values raise `QueryError::UnsupportedLoader("Unknown loader
'...'")`.

`"forge"` was added on top of the original four — when the server
returns `"forge"` the merger fetches the Forge loader metadata
(installer + processors honoured by `lighty-launch`) and folds the
server's extra libraries / args on top. No extra feature flag needed
— `lighty_updater` activates `forge` at the workspace level.

## Security notes

- Serve over HTTPS.
- Always populate `sha1` so the launcher can verify each library.
- Consider rate limiting + API key headers if your endpoint is public.

## Reference

Full server-side documentation, deployment recipes and reference
implementation:

- [LightyUpdater GitHub repository](https://github.com/Lighty-Launcher/LightyUpdater)

## See also

- [`../../../version/docs/how-to-use.md`](../../../version/docs/how-to-use.md)
  — `LightyVersionBuilder` reference
- [`../traits.md`](../traits.md) — `VersionInfo` requirements
- [`../query.md`](../query.md) — implementing a server-side loader
