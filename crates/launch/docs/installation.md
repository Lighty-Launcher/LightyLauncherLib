# Installation Process

## Overview

The installation system downloads and verifies all game files required to launch Minecraft. It uses parallel downloads, SHA1 verification, and automatic retry logic.

## Installation Architecture

```
Launch runner (execute_launch)
└─> Installer Trait (lighty_launch::installer::Installer)
    │   Phase 0: resolve_extra_mods (merges modpack + user mods into Version.mods)
    ├─> [optional] Modpack pipeline              (feature = "modrinth" | "curseforge")
    │   ├─> Resolve archive URL (Modrinth / CurseForge)
    │   ├─> Download .mrpack / .zip into cache (idempotent SHA1 marker)
    │   ├─> Extract archive
    │   ├─> Parse manifest (modrinth.index.json / manifest.json)
    │   ├─> Convert files to `Mods` entries (whitelist Modrinth URLs)
    │   └─> Copy overrides/ into runtime_dir (SkipWarn on conflicts)
    ├─> [optional] User-mods resolver            (feature = "modrinth"|"curseforge")
    │   └─> `lighty_modsloader::resolver::resolve` — BFS, dedup by ModKey,
    │       emits `ModloaderEvent::Resolve*` natively (feature = "events")
    │
    │   Phases 1-3: standard downloads (parallel — 8 buckets)
    ├─> Libraries Installation
    ├─> Natives Installation (download + extract)
    ├─> Client JAR Installation
    ├─> Assets Installation
    ├─> Mods Installation                     (subdir: mods/)
    ├─> Resource Packs Installation           (subdir: resourcepacks/)
    ├─> Shader Packs Installation             (subdir: shaderpacks/)
    └─> Datapacks Installation                (subdir: datapacks/)
```

`Installer::install` is implemented for any `T: VersionInfo<LoaderType =
Loader> + WithMods`. The runner is a pure caller — no pre-merge logic
outside of `install()`.

### Asset partitioning

The four mod-like buckets (mods / resourcepacks / shaderpacks /
datapacks) all share a single helper located in
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

Each public bucket (`mods.rs`, `resourcepacks.rs`, `shaderpacks.rs`,
`datapacks.rs`) is a thin wrapper that pins a fixed `subdir` prefix and
delegates to `collect` + `download`. Only `mods` enables
`legacy_fallback` (unqualified `path = filename` is treated as
`mods/<filename>` with a deprecation warn during the migration window
documented in `ASSETS_ROUTING.md`). The other buckets only accept a
fully-qualified path matching their own prefix.

The installer never builds `runtime_dir().join("mods")` unconditionally
anymore — each bucket places files under its own subdir, computed once
inside `collect`.

## Installation Phases

### Phase 1: Verification (Collect Tasks)

**Purpose**: Determine which files need to be downloaded

The orchestrator fans out an 8-way `tokio::join!` so every bucket walks
the metadata concurrently. The four mod-like buckets each return their
own `(tasks, bytes)` pair — the byte counter is computed inline by
`asset_partition::collect` so the orchestrator never has to re-scan the
`Mods` slice:

```rust
let mods_slice = builder.mods.as_deref().unwrap_or(&[]);
let (
    library_tasks,
    client_task,
    asset_tasks,
    (mod_tasks, mod_bytes),
    (resourcepack_tasks, resourcepack_bytes),
    (shaderpack_tasks, shaderpack_bytes),
    (datapack_tasks, datapack_bytes),
    (native_download_tasks, native_extract_paths),
) = tokio::join!(
    libraries::collect_library_tasks(self, &builder.libraries),
    client::collect_client_task(self, builder.client.as_ref()),
    assets::collect_asset_tasks(self, builder.assets.as_ref()),
    mods::collect_mod_tasks(self, mods_slice),
    resourcepacks::collect_resourcepack_tasks(self, mods_slice),
    shaderpacks::collect_shaderpack_tasks(self, mods_slice),
    datapacks::collect_datapack_tasks(self, mods_slice),
    natives::collect_native_tasks(self, builder.natives.as_deref().unwrap_or(&[])),
);
```

**What happens**:
- For each file type:
  1. Check if file exists on disk
  2. If exists, verify SHA1 hash
  3. If missing or hash mismatch → add to task list
  4. If valid → skip

**Example task**:
```rust
pub struct DownloadTask {
    pub url: String,
    pub path: PathBuf,
    pub sha1: Option<String>,
    pub size: Option<u64>,
}
```

### Phase 2: Decision

**Skip installation** if all files are valid:
```rust
if total_downloads == 0 {
    // Emit IsInstalled event
    // Extract natives (always required)
    // Return early
}
```

**Proceed with installation** if files need downloading:
```rust
// Emit InstallStarted event
// Execute parallel downloads
// Emit InstallCompleted event
```

### Phase 3: Parallel Download

All 8 buckets download concurrently through a single `tokio::try_join!`.
The mod-like buckets each receive the byte count returned by their own
`collect` step so per-bucket events stay accurate:

```rust
tokio::try_join!(
    libraries::download_libraries(library_tasks, event_bus),
    natives::download_and_extract_natives(self, native_download_tasks, native_extract_paths, event_bus),
    mods::download_mods(mod_tasks, event_bus),
    resourcepacks::download_resourcepacks(resourcepack_tasks, resourcepack_bytes, event_bus),
    shaderpacks::download_shaderpacks(shaderpack_tasks, shaderpack_bytes, event_bus),
    datapacks::download_datapacks(datapack_tasks, datapack_bytes, event_bus),
    client::download_client(client_task, event_bus),
    assets::download_assets(asset_tasks, event_bus),
)?;
```

### `calculate_download_size`

`calculate_download_size` only walks the metadata for libraries, client
JAR, assets and natives. The mod-like total is passed in as a single
pre-summed `mod_like_bytes: u64` produced by the four bucket
collectors — replacing the previous `O(N*M)` re-scan of four separate
`Mods` slices:

```rust
#[cfg(feature = "events")]
fn calculate_download_size(
    builder: &Version,
    library_tasks: &[(String, PathBuf)],
    client_task: &Option<(String, PathBuf)>,
    asset_tasks: &[(String, PathBuf)],
    native_download_tasks: &[(String, PathBuf)],
    mod_like_bytes: u64,
) -> u64;
```

The orchestrator computes `mod_like_bytes = mod_bytes + resourcepack_bytes
+ shaderpack_bytes + datapack_bytes` before calling it.

## Installation Components

### 1. Libraries

**Purpose**: Java dependencies (JARs) required by the game

**Location**: `{game_dir}/libraries/`

**Structure**:
```
libraries/
├── com/mojang/logging/1.0.0/logging-1.0.0.jar
├── org/lwjgl/lwjgl/3.3.3/lwjgl-3.3.3.jar
├── org/lwjgl/lwjgl-glfw/3.3.3/lwjgl-glfw-3.3.3.jar
└── net/fabricmc/fabric-loader/0.16.9/fabric-loader-0.16.9.jar
```

**Metadata example**:
```json
{
  "name": "org.lwjgl:lwjgl:3.3.3",
  "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl/3.3.3/lwjgl-3.3.3.jar",
  "sha1": "4158d7bf99b95428c5e8a8eb8d5d31e2f3f5c6a1",
  "size": 746293
}
```

**Download process**:
```rust
async fn download_libraries(
    tasks: Vec<(String, PathBuf)>,
    event_bus: Option<&EventBus>,
) -> InstallerResult<()> {
    if tasks.is_empty() {
        return Ok(());
    }

    // Per-file progress flows through the shared downloader, which emits
    // `LaunchEvent::InstallProgress { bytes }` increments globally.
    download_with_concurrency_limit(tasks, event_bus).await?;

    Ok(())
}
```

**Typical count**: 100-300 libraries depending on loader

### 2. Natives

**Purpose**: Platform-specific native binaries (LWJGL, OpenAL, etc.)

**Location**:
- Downloaded to: `{game_dir}/natives/`
- Extracted to: `{temp}/natives-{timestamp}/`

**Platform-specific**:
```
Windows:  lwjgl-3.3.3-natives-windows.jar
Linux:    lwjgl-3.3.3-natives-linux.jar
macOS:    lwjgl-3.3.3-natives-macos.jar
```

**Metadata example**:
```json
{
  "name": "org.lwjgl:lwjgl:3.3.3:natives-windows",
  "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl/3.3.3/lwjgl-3.3.3-natives-windows.jar",
  "sha1": "2b6166b5c1bc8b0c5e5c4b8e8f5e6a1c9d8e7f6a",
  "size": 142857,
  "extract": {
    "exclude": ["META-INF/"]
  }
}
```

**Download and extraction**:
```rust
async fn download_and_extract_natives(
    version: &impl VersionInfo,
    download_tasks: Vec<(String, PathBuf)>,
    extract_paths: Vec<PathBuf>,
    event_bus: Option<&EventBus>,
) -> InstallerResult<()> {
    // 1. Download native JARs if needed
    if !download_tasks.is_empty() {
        for (url, path) in download_tasks {
            download_file(&url, &path).await?;
        }
    }

    // 2. Extract natives to temporary directory
    let natives_dir = create_natives_temp_dir();

    for jar_path in extract_paths {
        extract_jar_excluding(&jar_path, &natives_dir, &["META-INF/"]).await?;
    }

    Ok(())
}
```

**Why extract every time?**
- Ensures clean state
- Prevents conflicts between runs
- Handles platform-specific libraries correctly

**Extraction rules**:
- Extract all `.dll` (Windows), `.so` (Linux), `.dylib` (macOS) files
- Exclude `META-INF/` directory (metadata, not needed)
- Flatten directory structure (all files in root)

**Typical count**: 5-15 native libraries

### 3. Client JAR

**Purpose**: Main Minecraft executable

**Location**: `{game_dir}/versions/{version}/{version}.jar`

**Example**:
```
versions/1.21.1/1.21.1.jar
```

**Metadata**:
```json
{
  "url": "https://piston-data.mojang.com/v1/objects/59353fb40c36d304f2035d51e7d6e6baa98dc05c/client.jar",
  "sha1": "59353fb40c36d304f2035d51e7d6e6baa98dc05c",
  "size": 26354187
}
```

**Download process**:
```rust
async fn download_client(
    task: Option<(String, PathBuf)>,
    event_bus: Option<&EventBus>,
) -> InstallerResult<()> {
    if let Some((url, path)) = task {
        download_with_concurrency_limit(vec![(url, path)], event_bus).await?;
    }
    Ok(())
}
```

**Size**: Typically 20-30 MB

### 4. Assets

**Purpose**: Game resources (textures, sounds, language files)

**Location**: `{game_dir}/assets/objects/`

**Structure** (hash-based):
```
assets/
├── indexes/
│   └── 16.json              # Asset index
└── objects/
    ├── 00/
    │   └── 001234abcd...    # Hashed asset file
    ├── 01/
    │   └── 015678efgh...
    └── ff/
        └── ffabcdef01...
```

**Asset index example** (`assets/indexes/16.json`):
```json
{
  "objects": {
    "minecraft/sounds/ambient/cave/cave1.ogg": {
      "hash": "f8c4b5e6a1d2c3b4a5e6f7d8c9b0a1e2f3d4c5b6",
      "size": 18357
    },
    "minecraft/textures/block/stone.png": {
      "hash": "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0",
      "size": 2048
    }
  }
}
```

**Download process**:
```rust
async fn download_assets(
    tasks: Vec<(String, PathBuf)>,
    event_bus: Option<&EventBus>,
) -> InstallerResult<()> {
    if tasks.is_empty() {
        return Ok(());
    }

    // Bounded-concurrency downloader streams `LaunchEvent::InstallProgress`
    // for each downloaded chunk; no per-batch event is emitted here.
    download_with_concurrency_limit(tasks, event_bus).await?;

    Ok(())
}
```

**Asset URL format**:
```
https://resources.download.minecraft.net/{hash[0:2]}/{hash}
```

**Example**:
```
Hash: f8c4b5e6a1d2c3b4a5e6f7d8c9b0a1e2f3d4c5b6
URL:  https://resources.download.minecraft.net/f8/f8c4b5e6a1d2c3b4a5e6f7d8c9b0a1e2f3d4c5b6
Path: assets/objects/f8/f8c4b5e6a1d2c3b4a5e6f7d8c9b0a1e2f3d4c5b6
```

**Typical count**: 3,000-10,000 assets

### 5. Mods

**Purpose**: Modifications for Fabric/Quilt/NeoForge

**Location**: `{game_dir}/mods/`

**Structure**:
```
mods/
├── fabric-api-0.100.0+1.21.jar
├── sodium-fabric-mc1.21-0.5.8.jar
└── iris-mc1.21-1.7.0.jar
```

**Metadata** (from LightyUpdater server):
```json
{
  "name": "fabric-api",
  "url": "https://server.com/mods/fabric-api-0.100.0.jar",
  "sha1": "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0",
  "size": 2450123,
  "enabled": true
}
```

**Download process**:
```rust
async fn download_mods(
    tasks: Vec<(String, PathBuf)>,
    event_bus: Option<&EventBus>,
) -> InstallerResult<()> {
    // Delegates to `asset_partition::download` with `label = "mods"`.
    // Each downloaded chunk contributes to the global
    // `LaunchEvent::InstallProgress { bytes }` counter.
    asset_partition::download(tasks, "mods", event_bus).await
}
```

**Mod management**:
- Disabled mods: Add `.disabled` suffix
- Remove old mods: Delete unlisted files
- Update mods: Replace if SHA1 mismatch

**Typical count**: 10-200 mods depending on modpack

### 6. Modpack (optional pre-step)

**Feature**: rides on `modrinth` and/or `curseforge` (no separate `modpack` feature — enabling a provider activates its modpack format parser)

**Purpose**: Install a community-built collection of mods + config overrides described by a `.mrpack` (Modrinth) or `.zip` (CurseForge) archive — runs **before** the user-mods resolver so its files end up in `Version.mods` for the standard download pipeline.

**Lives in**: `crates/launch/src/installer/ressources/modpack.rs` (download/extract/overrides). The manifest parsing + URL resolution + Modrinth whitelist live in `crates/modsloader/src/modpack/`.

**Pipeline** (`process()` in `modpack.rs`):

1. **Resolve archive URL**
   - `ModpackSource::ModrinthUrl(url)` → returned as-is
   - `ModpackSource::ModrinthPinned { project, version }` → Modrinth API call
   - `ModpackSource::CurseForgePinned { project_id, file_id }` → CurseForge API call (needs `set_api_key`)
2. **Cache lookup**: `<cache_dir>/modpacks/<url_sha1>.archive` + `<url_sha1>.installed` marker. If the marker matches the URL hash → skip everything (idempotent).
3. **Download archive** via `lighty_core::download::download_file_untracked` if not already cached.
4. **Extract** into `<cache_dir>/modpacks/work-<sha1>/` via `tokio::task::spawn_blocking` + the `zip` crate.
5. **Parse manifest** — dispatch on `modrinth.index.json` vs `manifest.json`.
6. **Reconcile loader/MC**: log `trace_warn!` if the manifest declares values different from the `VersionBuilder` (the builder's values stay authoritative in this first iteration; a future patch may flip the precedence).
7. **Convert files → `Vec<Mods>`**:
   - **Modrinth**: every `files[]` entry with `env.client == "required"`. **Each `downloads[0]` URL is checked against the whitelist** (`cdn.modrinth.com`, `*.fabricmc.net`, `*.forgecdn.net`, `*.neoforged.net`, `maven.minecraftforge.net`, `github.com`/`raw.githubusercontent.com`/`gist.githubusercontent.com`, `gitlab.com`). HTTPS only. Failures return `QueryError::Conversion`.
   - **CurseForge**: each `(projectID, fileID)` is resolved via `fetch_pinned_file()` to obtain the download URL. If `download_url` is null (third-party distribution disabled), the install fails fast with `QueryError::ModDistributionForbidden`.
8. **Extract `overrides/`** (and `client-overrides/` on Modrinth) into `version.runtime_dir()` recursively. **Existing user files are NEVER overwritten** — they're kept and the override is skipped with `trace_warn!`. Returns the count of files actually copied.
9. **Marker file** written with the URL SHA1 so subsequent runs are no-ops.
10. **Cleanup** `work-<sha1>/` directory.

**Output**: `Vec<Mods>` merged into the pivot **before** the user-mod resolver. Modpack files go first so a user-attached mod with the same filename wins the SHA1 check downstream.

**Whitelist enforcement**: implemented in `lighty_modsloader::modpack::whitelist::validate_modrinth_download_url`. Strict — any URL pointing outside the official Modrinth host list fails the install.

**Events** (with `events` feature) — now live under `ModloaderEvent`,
not `LaunchEvent`:
```rust
ModloaderEvent::ModpackResolveStart        { source: String }
ModloaderEvent::ModpackArchiveDownloaded   { sha1: String, bytes: u64 }
ModloaderEvent::ModpackOverridesExtracted  { count: usize }
ModloaderEvent::ModpackInstalled           { name: String, mods_count: usize }
```

**Cache layout**:
```
{cache_dir}/modpacks/
├── 4f2c…1a8b.archive       # downloaded .mrpack / .zip
├── 4f2c…1a8b.installed     # marker (contents = URL SHA1)
└── work-4f2c…1a8b/         # transient extraction dir (cleaned after install)
```

**Idempotence**: relaunching the same instance does **not** re-download or re-extract the archive — the marker check short-circuits before any I/O.

**Force re-install**: delete `{cache_dir}/modpacks/<sha1>.installed` (or wipe the whole `modpacks/` directory) to force a fresh extract on the next run.

## SHA1 Verification

**Purpose**: Ensure file integrity and avoid re-downloading

```rust
async fn verify_sha1(path: &Path, expected: &str) -> bool {
    use sha1::{Sha1, Digest};

    let mut file = match File::open(path).await {
        Ok(f) => f,
        Err(_) => return false,
    };

    let mut hasher = Sha1::new();
    let mut buffer = vec![0u8; 8192];

    loop {
        let n = match file.read(&mut buffer).await {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => return false,
        };

        hasher.update(&buffer[..n]);
    }

    let hash = format!("{:x}", hasher.finalize());
    hash == expected
}
```

**When used**:
- Before download: Skip if file exists and SHA1 matches
- After download: Verify downloaded file (optional, based on metadata availability)

**Benefits**:
- Saves bandwidth (skip already-downloaded files)
- Ensures file integrity
- Detects corrupted downloads

## Download Implementation

### File Download with Retry

```rust
async fn download_file(url: &str, path: &Path) -> InstallerResult<()> {
    const MAX_RETRIES: u32 = 3;

    for attempt in 1..=MAX_RETRIES {
        match try_download(url, path).await {
            Ok(_) => return Ok(()),
            Err(e) if attempt < MAX_RETRIES => {
                lighty_core::trace_warn!(
                    "Download failed (attempt {}/{}): {}",
                    attempt, MAX_RETRIES, e
                );
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            Err(e) => return Err(InstallerError::DownloadFailed(e.to_string())),
        }
    }

    unreachable!()
}

async fn try_download(url: &str, path: &Path) -> Result<(), Box<dyn Error>> {
    // Create parent directory
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    // Download file
    let response = reqwest::get(url).await?.error_for_status()?;
    let bytes = response.bytes().await?;

    // Write to disk
    tokio::fs::write(path, bytes).await?;

    Ok(())
}
```

### Progress Tracking

With events feature, the shared downloader emits a single global
`LaunchEvent::InstallProgress { bytes }` per chunk written across **all
eight buckets**. The `bytes` field is a delta — sum it client-side and
compare against the `total_bytes` reported by `InstallStarted` to
render a progress bar:

```rust
#[cfg(feature = "events")]
pub async fn download_with_progress(
    url: &str,
    path: &Path,
    event_bus: &EventBus,
) -> InstallerResult<()> {
    let response = reqwest::get(url).await?.error_for_status()?;
    let mut file = File::create(path).await?;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;

        event_bus.emit(Event::Launch(LaunchEvent::InstallProgress {
            bytes: chunk.len() as u64,
        }));
    }

    Ok(())
}
```

Per-bucket finalisation events (resource packs, shader packs,
datapacks) live under `ModloaderEvent` — see the
[events documentation](./events.md) for the list.

## Directory Creation

Directories are created on-demand:
```rust
async fn create_directories(version: &impl VersionInfo) {
    let parent_path = version.game_dirs().to_path_buf();

    mkdir!(parent_path.join("libraries"));
    mkdir!(parent_path.join("natives"));
    mkdir!(parent_path.join("assets").join("objects"));
}
```

**Created directories**:
```
{game_dir}/
├── libraries/          # Java JAR libraries
├── natives/            # Native library downloads
├── assets/
│   ├── indexes/        # Asset index files
│   └── objects/        # Hashed asset files
├── mods/              # Mod files (if applicable)
├── versions/          # Client JAR
├── saves/             # World saves (created by game)
├── resourcepacks/     # Resource packs (created by game)
└── screenshots/       # Screenshots (created by game)
```

## Events Emitted

### Installation Events (`LaunchEvent`)

The installer-side `LaunchEvent` variants are intentionally minimal —
per-bucket progress is reported through the global `InstallProgress`
byte counter, not per-file callbacks:

```rust
LaunchEvent::IsInstalled       { version: String }
LaunchEvent::InstallStarted    { version: String, total_bytes: u64 }
LaunchEvent::InstallProgress   { bytes: u64 }
LaunchEvent::InstallCompleted  { version: String, total_bytes: u64 }
```

### Mod-source / modpack events (`ModloaderEvent`)

The dependency resolver, the modpack pipeline and the three new
mod-like buckets emit through `ModloaderEvent` (see
[crates/event/docs/events.md](../../event/docs/events.md)):

```rust
ModloaderEvent::ResolveStarted          { request_count: usize }
ModloaderEvent::ResolveFetching         { source: String, identifier: String }
ModloaderEvent::ResolveDependency       { parent: String, dependency: String }
ModloaderEvent::ResolveCompleted        { total_mods: usize }

ModloaderEvent::ModpackResolveStart       { source: String }
ModloaderEvent::ModpackArchiveDownloaded  { sha1: String, bytes: u64 }
ModloaderEvent::ModpackOverridesExtracted { count: usize }
ModloaderEvent::ModpackInstalled          { name: String, mods_count: usize }

ModloaderEvent::ResourcePacksInstalled { count: usize, bytes: u64 }
ModloaderEvent::ShaderPacksInstalled   { count: usize, bytes: u64 }
ModloaderEvent::DatapacksInstalled     { count: usize, bytes: u64 }
```

## Performance Characteristics

### Parallel Downloads
- **Libraries**: Downloaded sequentially (100-300 files, ~50-100 MB total)
- **Natives**: Downloaded sequentially (5-15 files, ~5-10 MB total)
- **Client**: Single file (~20-30 MB)
- **Assets**: Downloaded in batches of 50 (3000-10000 files, ~200-500 MB total)
- **Mods**: Downloaded sequentially (10-200 files, variable size)

All categories run in parallel using `tokio::try_join!`.

### Optimization Strategies

1. **Skip verified files**: SHA1 check before download
2. **Batch asset downloads**: 50 assets per batch
3. **Concurrent categories**: All types download simultaneously
4. **Automatic retry**: 3 attempts per file
5. **Temp directory for natives**: Clean state per launch

## Error Handling

```rust
pub enum InstallerError {
    DownloadFailed(String),
    VerificationFailed(String),
    ExtractionFailed(String),
    IOError(std::io::Error),
}
```

**Example**:
```rust
match version.install(metadata, event_bus).await {
    Ok(_) => println!("Installation complete"),
    Err(InstallerError::DownloadFailed(url)) => {
        eprintln!("Failed to download: {}", url);
    }
    Err(InstallerError::VerificationFailed(file)) => {
        eprintln!("Verification failed: {}", file);
    }
    Err(e) => eprintln!("Installation error: {}", e),
}
```

## Complete Example

```rust
use lighty_core::AppState;
use lighty_launcher::prelude::*;
use lighty_launch::Installer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("MyLauncher")?;

    let instance = VersionBuilder::new(
        "fabric-1.21",
        Loader::Fabric,
        "0.16.9",
        "1.21.1",
    );

    // Get metadata
    let metadata = instance.get_metadata().await?;
    let version = match metadata.as_ref() {
        VersionMetaData::Version(v) => v,
        _ => return Err(anyhow::anyhow!("Invalid metadata")),
    };

    // Install all dependencies
    instance.install(version, None).await?;

    println!("Installation complete!");

    Ok(())
}
```

## Related Documentation

- [Launch Process](./launch.md) - Complete launch flow
- [Events](./events.md) - Event types
- [How to Use](./how-to-use.md) - Practical examples
