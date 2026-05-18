# Installation

The install pipeline downloads and verifies every game file before the
JVM is spawned. It's 8-way parallel, idempotent (SHA1 verification),
and emits per-bucket events when the `events` feature is on.

## Pipeline overview

```text
Installer::install(&self, &Version, event_bus)
├── Phase 0: resolve_extra_mods
│   ├── [optional] Modpack pipeline           (feature = "modrinth" | "curseforge")
│   │   └─► Vec<Mods> merged first (user mods win on filename dedup)
│   └── [optional] User-mod resolver           (feature = "modrinth" | "curseforge")
│       └─► lighty_modsloader::resolver::resolve  — BFS, dedup by ModKey
│
├── Phase 1: Verification
│   └── 8-way tokio::join! — every bucket walks its slice concurrently
│       Each bucket returns (Vec<(url, path)>, bytes) for missing/outdated files
│
├── Phase 2: Decision
│   └── If total_downloads == 0 → emit IsInstalled, re-extract natives, return
│
└── Phase 3: Parallel download
    └── 8-way tokio::try_join! — libraries, natives, client, assets,
        mods, resourcepacks, shaderpacks, datapacks
```

The orchestrator lives in
`crates/launch/src/installer/installer.rs`. `Installer::install` is
blanket-implemented for any `T: VersionInfo<LoaderType = Loader> + WithMods`.

## The eight buckets

| Bucket | Layout | Source | Notes |
|---|---|---|---|
| **Libraries** | `{game_dir}/libraries/<maven-path>` | loader metadata | 100-300 files; SHA1 verified |
| **Natives** | `{game_dir}/natives/` + extracted to temp dir per launch | loader metadata | LWJGL `.dll` / `.so` / `.dylib`; extracted every launch (clean state) |
| **Client** | `{game_dir}/versions/{version}/{version}.jar` | loader metadata | ~20-30 MB |
| **Assets** | `{game_dir}/assets/objects/<aa>/<hash>` | Mojang asset index | 3 000 – 10 000 files |
| **Mods** | `{runtime_dir}/mods/` | LightyUpdater + Modrinth/CurseForge resolver | Legacy fallback: unqualified `path = filename` mapped to `mods/<filename>` |
| **Resource packs** | `{runtime_dir}/resourcepacks/` | mods slice (qualified path) | Strict prefix match |
| **Shader packs** | `{runtime_dir}/shaderpacks/` | mods slice (qualified path) | Strict prefix match |
| **Datapacks** | `{runtime_dir}/datapacks/` | mods slice (qualified path) | Strict prefix match |

All four mod-like buckets share a single helper at
`crates/launch/src/installer/ressources/asset_partition.rs`:

```rust
pub(super) async fn collect<V: VersionInfo>(
    version: &V,
    mods: &[Mods],
    subdir: &str,
    legacy_fallback: bool,
) -> (Vec<(String, PathBuf)>, u64);

pub(super) async fn download(
    tasks: Vec<(String, PathBuf)>,
    label: &str,
    #[cfg(feature = "events")] event_bus: Option<&EventBus>,
) -> InstallerResult<()>;
```

Each bucket file (`mods.rs`, `resourcepacks.rs`, …) is a thin wrapper
pinning its own `subdir` prefix. Only `mods` enables `legacy_fallback`
— the routing migration is documented in `ASSETS_ROUTING.md` at the
workspace root.

## SHA1 verification

Before any download, the verifier reads the file at the destination
path and compares the SHA1 against the metadata value. Three outcomes:

- **Missing** → enqueue download.
- **SHA1 mismatch** → enqueue redownload.
- **Match** → skip.

When every bucket comes back with zero tasks, the installer emits
`LaunchEvent::IsInstalled` and short-circuits. Natives are always
re-extracted into a fresh temp dir per launch (the JVM unpacks them
from a clean directory each time to avoid LWJGL conflicts).

## Modpack pre-step (optional)

Lives in `crates/launch/src/installer/ressources/modpack.rs`. Runs
**before** the user-mod resolver, so its files end up in `Version.mods`
in time for the standard download pipeline. Activated by the
`modrinth` and / or `curseforge` Cargo feature (no separate `modpack`
feature — enabling a provider activates its modpack format parser).

Pipeline:

1. **Resolve archive URL** — `ModrinthUrl`, `ModrinthPinned { project,
   version }` (API call), or `CurseForgePinned { project_id, file_id }`
   (API call, needs `set_api_key`).
2. **Cache lookup** — `<cache_dir>/modpacks/<url_sha1>.archive` +
   `.installed` marker. Marker match → skip everything (idempotent).
3. **Download archive** (if cache miss).
4. **Extract** into `<cache_dir>/modpacks/work-<sha1>/`.
5. **Parse manifest** — `modrinth.index.json` vs `manifest.json`.
6. **Reconcile loader / MC version** with the `VersionBuilder`
   (builder wins; mismatches log a `trace_warn`).
7. **Convert files → `Vec<Mods>`**:
   - **Modrinth**: every `files[]` entry with `env.client == "required"`.
   - **CurseForge**: resolve each `(projectID, fileID)` via the API.
     If `download_url` is null (third-party distribution disabled),
     fail fast with `QueryError::ModDistributionForbidden`.
8. **Extract `overrides/`** (and `client-overrides/` on Modrinth) into
   `version.runtime_dir()`. **Existing user files are never
   overwritten** — they're kept and the override is skipped with a
   warning.
9. Write the `.installed` marker so subsequent runs no-op.
10. Clean up the `work-<sha1>/` directory.

Force re-install: delete
`{cache_dir}/modpacks/<sha1>.installed` (or wipe `modpacks/`).

Modpack events (with `events` feature) live under `ModloaderEvent` —
see [events.md](./events.md#modloaderevent-variants).

## Download implementation

Single shared downloader in `lighty-core` with bounded concurrency,
3 retries per file, and chunked writes. When the `events` feature is
on, each chunk emits `LaunchEvent::InstallProgress { bytes }` — clients
sum that against `total_bytes` from `InstallStarted` to drive a
progress bar.

## Total-byte calculation

`calculate_download_size` only walks metadata for libraries, client
JAR, assets and natives. The mod-like total is a single pre-summed
`mod_like_bytes: u64` returned by the four bucket collectors —
avoiding an O(N·M) re-scan of the `Mods` slice:

```rust
let mod_like = mod_bytes
             + resourcepack_bytes
             + shaderpack_bytes
             + datapack_bytes;
```

## Loader-specific post-install

`Installer::install` only handles the universal pipeline. Loaders that
need a post-step do it from `execute_launch` *after* `install()`:

- **Forge** (1.13+) — download `install_profile` libraries, then run
  Forge install processors via a `Java -jar` exec.
- **Forge legacy** (1.7.10 – 1.12.2) — no processors; extract the
  bundled universal JAR to its Maven path so the classpath resolves.
- **NeoForge** — same shape as modern Forge.

The Cargo `forge` feature covers **both** modern and legacy Forge.
Per-loader processor mechanics are documented in
[`crates/loaders/docs/loaders/forge.md`](../../loaders/docs/loaders/forge.md)
and [`neoforge.md`](../../loaders/docs/loaders/neoforge.md).

## Directories created on demand

```text
{game_dir}/
├── libraries/
├── natives/
├── assets/{indexes,objects}/
├── versions/<version>/
├── mods/                    (runtime_dir)
├── resourcepacks/           (runtime_dir)
├── shaderpacks/             (runtime_dir)
└── datapacks/               (runtime_dir)
```

`runtime_dir()` defaults to `game_dirs()` but the runner may rewrite it
via `set_runtime_dir()` when the caller overrides `KEY_GAME_DIRECTORY`
on the `ArgumentsBuilder` (relative override resolves under
`game_dirs`, absolute override wins outright).

## Standalone install

```rust
# use lighty_core::AppState;
# use lighty_launch::errors::InstallerResult;
# use lighty_loaders::types::Loader;
# use lighty_loaders::types::version_metadata::VersionMetaData;
# use lighty_version::VersionBuilder;
use lighty_launch::installer::Installer;

# async fn run() -> InstallerResult<()> {
# AppState::init("MyLauncher").ok();
let mut instance = VersionBuilder::new("inst", Loader::Fabric, "0.16.9", "1.21.1");
let metadata = instance.get_metadata().await?;
if let VersionMetaData::Version(v) = metadata.as_ref() {
    instance.install(v, #[cfg(feature = "events")] None).await?;
}
# Ok(()) }
```

## Errors

```rust
pub enum InstallerError {
    DownloadFailed(String),
    VerificationFailed(String),
    ExtractionFailed(String),
    InvalidMetadata,
    NoPid,
    IOError(std::io::Error),
    // …
}
```

## Related

- [Launch](./launch.md) — where `install()` sits in the pipeline
- [Events](./events.md) — `LaunchEvent::Install*` + `ModloaderEvent::*`
- [Arguments](./arguments.md) — `${classpath}` build, `KEY_GAME_DIRECTORY` override
- Loaders: [`crates/loaders/docs/loaders/`](../../loaders/docs/loaders/)
- Mods / modpacks: [`crates/modsloader/docs/`](../../modsloader/docs/)
