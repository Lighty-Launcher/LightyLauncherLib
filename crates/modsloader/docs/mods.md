# Mods

Pin individual mods, resourcepacks, shaderpacks or datapacks from
Modrinth or CurseForge. Combine with [`modpacks.md`](./modpacks.md) for
the full pack flow.

## API surface

```rust
.with_mod()
    .with_modrinth_mods(vec![
        ("sodium", None),                  // mod
        ("sodium-extra", None),            // resourcepack — routed automatically
        ("complementary-reimagined", None) // shader     — routed automatically
    ])
    .with_curseforge_mods(vec![
        (238222, None),  // JEI (mod)
        (393402, None),  // Pixel Daydream (resourcepack) — routed automatically
    ])
    .done()
```

Both methods take a list of tuples. The second element is an optional
pin — `None` lets the resolver pick the latest release compatible with
the instance's `(minecraft_version, loader)`. Required dependencies are
followed transitively and deduplicated by `(source, project)`.

## Asset routing — where the file lands

`Mods.path` is **qualified by sub-folder** and used **verbatim** by the
installer: `runtime_dir.join(mods.path)` with no extra prefix. The
provider clients compute the sub-folder at fetch time so each asset
lands in its idiomatic place under the instance.

| `path` emitted by the client | Final location |
|---|---|
| `mods/sodium-fabric-0.5.8.jar` | `<runtime>/mods/sodium-fabric-0.5.8.jar` |
| `resourcepacks/sodium-extra.zip` | `<runtime>/resourcepacks/sodium-extra.zip` |
| `shaderpacks/iris.zip` | `<runtime>/shaderpacks/iris.zip` |
| `datapacks/foo.zip` | `<runtime>/datapacks/foo.zip` |

### Modrinth — `project_type` → sub-folder

Before each `fetch`, the client does a `GET /project/{slug}` lookup to
read `project_type`, then maps:

| Modrinth `project_type` | Sub-folder |
|---|---|
| `mod` | `mods` |
| `resourcepack` | `resourcepacks` |
| `shader` | `shaderpacks` |
| `datapack` | `datapacks` (top-level — see warning below) |
| anything else | **hard error** `QueryError::UnsupportedFormat` |

The result is memoized in `PROJECT_TYPE_CACHE` (process-wide,
`Cache<String, Arc<String>>`, same TTL as the main `MODRINTH_CACHE`)
so the extra round-trip is paid once per project regardless of how many
versions are pulled.

> **Datapacks**: Minecraft datapacks live in `<world>/datapacks/`
> (per-world). The client routes them to a top-level `datapacks/`
> directory and emits a `trace_warn!` — proper per-world install
> requires manual handling and is only safe through a modpack's
> overrides targeting a specific world.

### CurseForge — `classId` → sub-folder

Same pattern via `GET /mods/{mod_id}` reading `classId`:

| CurseForge `classId` | Sub-folder |
|---|---|
| `6` (mod) | `mods` |
| `12` (resourcepack) | `resourcepacks` |
| `6552` (shaderpack) | `shaderpacks` |
| anything else | **hard error** `QueryError::UnsupportedFormat` |

Cached in `CLASS_ID_CACHE` (`Cache<u32, Arc<u32>>`). The helper
`lighty_modsloader::curseforge::client::install_subdir_for(mod_id, ttl)`
is exported for use by the modpack pipeline.

CurseForge has no `datapack` class — datapacks distributed there are
either bundled into a mod or shipped via World (`classId 17`), which is
out of scope.

## Pinning a specific version

### Modrinth — `version_id`

1. Open `https://modrinth.com/mod/<slug>/versions`.
2. Click the target version.
3. The URL becomes `.../mod/<slug>/version/<version_id>`.
4. The trailing segment is `version_id` — an **opaque string**
   (`"PpRTuoEh"`), **not** the human-readable version `"0.5.8"`.

```rust
.with_modrinth_mods(vec![
    ("sodium", Some("PpRTuoEh".into())),  // pinned
    ("lithium", None),                     // latest compatible
])
```

API alternative (read-only): `GET https://api.modrinth.com/v2/project/<slug>/version`
returns the list of versions with their `id`.

### CurseForge — `mod_id` + `file_id`

- **`mod_id`** is the "Project ID" shown in the About sidebar of
  `https://www.curseforge.com/minecraft/mc-mods/<slug>`. E.g. JEI = `238222`.
- **`file_id`** is the trailing URL segment after clicking a file under
  the Files tab: `https://www.curseforge.com/minecraft/mc-mods/<slug>/files/<file_id>`.

```rust
.with_curseforge_mods(vec![
    (238222, Some(5234567)),  // JEI pinned
    (238222, None),           // JEI latest
])
```

API alternative: `GET /v1/mods/<mod_id>/files` (requires `x-api-key`) returns the file list.

### Setting the CurseForge API key

CurseForge requires an API key. Set it once before any launch:

```rust
lighty_launcher::mods::curseforge::set_api_key(
    std::env::var("CURSEFORGE_API_KEY")?
);
```

(Implemented in `crates/modsloader/src/curseforge/api.rs`, re-exported
from the `curseforge` module.) Get a key at
<https://console.curseforge.com/?#/api-keys>.

## How resolution works

`lighty_modsloader::resolver::resolve` is a BFS over the user request
list:

1. Pop a request from the queue.
2. Skip if its `ModKey` (source + project id, version-agnostic) is
   already in `visited`.
3. Fetch the pivot `Mods` entry via the appropriate API client
   (`modrinth::fetch` or `curseforge::fetch`). The sub-folder lookup
   happens here.
4. Enqueue every `required` dependency the response declared.
5. Repeat until the queue is empty.

Output: `Vec<Mods>` ready for the standard mods installer.

### Provider notes

- **Modrinth** runs against the public Labrinth API
  (`https://api.modrinth.com/v2`). No key required; uses a custom
  `User-Agent` per Modrinth's guidance (defined in
  `modrinth/api.rs`).
- **CurseForge** runs against the Core API
  (`https://api.curseforge.com/v1`) with `x-api-key`. Some projects
  disable third-party distribution — those return `download_url: null`,
  surfaced as `QueryError::ModDistributionForbidden`.

### Loader compatibility

| Loader | Modrinth | CurseForge |
|---|---|---|
| Fabric | ✓ | ✓ |
| Forge | ✓ | ✓ |
| NeoForge | ✓ | ✓ |
| Quilt | ✓ | ✓ |
| Vanilla / OptiFine / LightyUpdater | rejected (`UnsupportedLoader`) | rejected |

## Errors you might see

| Variant | Meaning |
|---|---|
| `QueryError::ModNotFound { provider, id }` | Project / version not found by ID |
| `QueryError::ModIncompatible { provider, id, mc, loader }` | No release compatible with `(mc, loader)` |
| `QueryError::ModDistributionForbidden { id }` | CurseForge `download_url` is null |
| `QueryError::UnsupportedFormat { what, expected, found }` | `project_type` / `classId` not in the routing table |
| `QueryError::UnsupportedLoader(...)` | Vanilla / OptiFine / LightyUpdater used with a mod source |
| `QueryError::Network(...)` | Underlying HTTP failure |

All flow through `lighty_core::QueryError` (shared with loader-side errors).
