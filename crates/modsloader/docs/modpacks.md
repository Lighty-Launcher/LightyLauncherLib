# Modpacks

Install a pre-built collection of mods + config overrides described by
a Modrinth `.mrpack` or a CurseForge `.zip`. Compiled in as soon as the
matching provider feature (`modrinth` or `curseforge`) is enabled — no
separate `modpack` feature.

## API

```rust
.with_mod()
    .with_modrinth_modpack("https://cdn.modrinth.com/.../pack.mrpack")
    // or, pinned by (project, version_id):
    .with_modrinth_modpack(ModpackSource::ModrinthPinned {
        project: "simply-optimized".into(),
        version: Some("AbCdEfGh".into()),
    })
    // or CurseForge:
    .with_curseforge_modpack(project_id, file_id)
    .done()
```

One modpack per instance — calling `.with_*_modpack(...)` twice replaces
the previous source. Modpack + per-mod methods can be combined; modpack
files install first, user mods override on filename conflict.

`ModpackSource` lives in the flat module file
`crates/modsloader/src/modpack.rs` (it's just the enum + `From<&str>` /
`From<String>` impls — nothing else). The actual parsers live next to
their provider clients (cf. *Where the code lives* below).

## Format support

### Modrinth `.mrpack`

ZIP with at its root:

| Entry | Role |
|---|---|
| `modrinth.index.json` | Manifest (cf. below) |
| `overrides/` | Files to copy into `runtime_dir/` (configs, scripts, resourcepacks…) |
| `client-overrides/` *(opt)* | Client-only surcouche |
| `server-overrides/` *(opt)* | **Ignored** by the launcher |

`modrinth.index.json` (excerpt):

```json
{
  "formatVersion": 1,
  "game": "minecraft",
  "versionId": "1.0.0",
  "name": "My Pack",
  "files": [
    {
      "path": "mods/sodium-fabric-mc1.21-0.5.8.jar",
      "hashes": { "sha1": "abc...", "sha512": "..." },
      "env": { "client": "required", "server": "required" },
      "downloads": ["https://cdn.modrinth.com/.../sodium.jar"],
      "fileSize": 854321
    }
  ],
  "dependencies": {
    "minecraft": "1.21.1",
    "fabric-loader": "0.16.9"
  }
}
```

Only files with `env.client == "required"` (or no `env` block) are
installed. Each entry's `path` is **relative to `runtime_dir/`** — the
pack author controls where the file lands (`mods/foo.jar`,
`resourcepacks/bar.zip`, …). The installer no longer prefixes `mods/`
itself, so a manifest entry like `resourcepacks/foo.zip` now correctly
lands at `<runtime>/resourcepacks/foo.zip` (previously it was
mis-routed to `<runtime>/mods/resourcepacks/foo.zip`).

### CurseForge `.zip`

ZIP with at its root:

| Entry | Role |
|---|---|
| `manifest.json` | Manifest (cf. below) |
| `overrides/` | Surcouche directory (name configurable via `manifest.overrides`) |
| `modlist.html` | **Ignored** |

`manifest.json` (excerpt):

```json
{
  "minecraft": {
    "version": "1.21.1",
    "modLoaders": [{ "id": "fabric-0.16.9", "primary": true }]
  },
  "manifestType": "minecraftModpack",
  "manifestVersion": 1,
  "name": "My Pack",
  "version": "1.0.0",
  "files": [
    { "projectID": 238222, "fileID": 5234567, "required": true }
  ],
  "overrides": "overrides"
}
```

The launcher resolves each `(projectID, fileID)` through the CurseForge
API (so `set_api_key` must have been called) and rejects packs that
contain files where the project has disabled third-party distribution
(`ModDistributionForbidden`). Each resolved file is routed through
`curseforge::client::install_subdir_for` so resourcepacks / shaderpacks
declared inside a CF modpack land in their proper sub-folder instead of
being shoved into `mods/`.

The `id` in `modLoaders` is parsed as `<loader>-<version>` and matched
to `Loader::{Fabric, Forge, NeoForge, Quilt}`. Other loaders surface
`UnsupportedLoader`.

## Where the code lives

| Concern | Path |
|---|---|
| `ModpackSource` enum + `From` impls | `crates/modsloader/src/modpack.rs` |
| `.mrpack` URL resolver + manifest parser | `crates/modsloader/src/modrinth/modpack.rs` |
| `.mrpack` wire types (`MrpackManifest`, …) | `crates/modsloader/src/modrinth/modpack_metadata.rs` |
| `.zip` (CF) URL resolver + manifest parser | `crates/modsloader/src/curseforge/modpack.rs` |
| `.zip` (CF) wire types (`CfModpackManifest`, …) | `crates/modsloader/src/curseforge/modpack_metadata.rs` |
| Download / extract / overrides / cache | `crates/launch/src/installer/ressources/modpack/` |

The old `crates/modsloader/src/modpack/` directory no longer exists.
Parsers are now co-located with their provider client.

## Conflict policy

When the modpack extracts `overrides/` into `runtime_dir/`:

- **Existing files are kept.** The override is skipped and
  `trace_warn!("[Modpack] Skipping override of existing file: …")` is
  emitted.
- **Directories** are merged recursively.

Rationale: users who tweaked their `options.txt`, keybinds, or
NBT-backed configs don't want them silently wiped on relaunch. To
force-reset, delete `runtime_dir/` (or the conflicting files) manually
and rerun.

## Reconcile loader/MC

If the pack manifest declares an MC + loader different from the
`VersionBuilder`'s values, the launcher logs a `trace_warn!` line and
**uses the builder's values**. This is the current behaviour — a future
patch will likely flip the precedence (modpack authoritative). Either
way, the warn line gives you an audit trail.

## Cache + idempotence

Archives are cached at `<cache_dir>/modpacks/<url_sha1>.archive` with a
`<url_sha1>.installed` marker. On relaunch the marker is checked — if
it matches the SHA1 of the resolved URL, the whole pipeline short-
circuits.

Force a re-install:

```bash
rm "$XDG_CACHE_HOME"/LightyLauncher/modpacks/*.installed
# or wipe the whole cache:
rm -rf "$XDG_CACHE_HOME"/LightyLauncher/modpacks
```

## Events (`events` feature)

```rust
ModloaderEvent::ModpackResolveStart       { source: String }
ModloaderEvent::ModpackArchiveDownloaded  { sha1: String, bytes: u64 }
ModloaderEvent::ModpackOverridesExtracted { count: usize }    // reserved
ModloaderEvent::ModpackInstalled          { name: String, mods_count: usize }
```

See [`events.md`](./events.md) for the full list (including the new
`ResourcePacksInstalled` / `ShaderPacksInstalled` / `DatapacksInstalled`
bucket summaries).

## Errors

| Variant | Cause |
|---|---|
| `QueryError::Conversion { message: "Unsupported .mrpack formatVersion: …" }` | Format bump newer than 1 — upgrade the library |
| `QueryError::Conversion { message: "Modpack archive contains neither modrinth.index.json nor manifest.json" }` | Archive doesn't look like a known modpack |
| `QueryError::ModDistributionForbidden { id }` | A CurseForge file in the pack has `download_url: null` |
| `QueryError::UnsupportedFormat { what, expected, found }` | A CF modpack file references a `classId` outside `{6, 12, 6552}` |
| `QueryError::UnsupportedLoader(...)` | Pack's loader id doesn't map to Fabric / Forge / NeoForge / Quilt |
