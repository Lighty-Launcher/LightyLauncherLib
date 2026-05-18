# Using lighty-core

One short example per feature. Pick what you need; everything composes.

## 1. AppState — initialise once

```rust,no_run
use lighty_core::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("LightyLauncher")?;

    println!("data: {}", AppState::data_dir().display());
    println!("cache: {}", AppState::cache_dir().display());
    Ok(())
}
```

Path layout per OS and the full API are in
[`app_state.md`](./app_state.md).

## 2. Download a file

`download_file_untracked` for fire-and-forget, `download_file` when you
need progress.

```rust,no_run
use lighty_core::download::{download_file, download_file_untracked};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // No progress
    download_file_untracked(
        "https://example.com/file.bin",
        "/tmp/file.bin",
    ).await?;

    // With progress callback
    let bytes = download_file(
        "https://example.com/file.bin",
        |current, total| {
            if total > 0 {
                println!("{:.1}%", current as f64 * 100.0 / total as f64);
            }
        },
    ).await?;
    println!("got {} bytes", bytes.len());
    Ok(())
}
```

Both return `DownloadResult<_>`. The HTTP client is the shared
`hosts::HTTP_CLIENT` — connection pooling is automatic.

## 3. Extract an archive (with optional progress events)

`zip_extract` and `tar_gz_extract` enforce path-traversal protection
and reject files above 2 GiB.

```rust,no_run
use lighty_core::extract::zip_extract;
use tokio::{fs::File, io::BufReader};
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let archive = BufReader::new(File::open("mod.zip").await?);

    #[cfg(feature = "events")]
    zip_extract(archive, Path::new("./out"), None).await?;

    #[cfg(not(feature = "events"))]
    zip_extract(archive, Path::new("./out")).await?;
    Ok(())
}
```

With the `events` feature on, pass `Some(&event_bus)` to emit
`CoreEvent::Extraction*`. See [`events.md`](./events.md).

## 4. Verify a SHA1 hash

```rust,no_run
use lighty_core::hash::{verify_file_sha1, verify_file_sha1_streaming};
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let small = Path::new("mod.jar");
    let big   = Path::new("jre.tar.gz");

    // Small files (read into memory once)
    let ok = verify_file_sha1(small, "abc...").await?;

    // Large files (streaming, 8 KiB buffer)
    let ok_streaming = verify_file_sha1_streaming(big, "def...").await?;

    println!("{ok} / {ok_streaming}");
    Ok(())
}
```

Sync variants (`calculate_file_sha1_sync`, `verify_file_sha1_sync`)
exist for non-async contexts. `calculate_sha1_bytes` and
`calculate_sha1_bytes_raw` hash in-memory data. Full surface in
[`hash.md`](./hash.md).

## 5. Detect OS / architecture at compile time

```rust,no_run
use lighty_core::system::{OS, ARCHITECTURE, OperatingSystem};

fn pick_archive_ext() -> &'static str {
    OS.get_archive_type().unwrap_or("zip")   // "zip" on Windows, "tar.gz" elsewhere
}

fn main() {
    println!("os: {OS}");
    println!("arch: {ARCHITECTURE}");
    if matches!(OS, OperatingSystem::OSX) {
        println!("running on macOS");
    }
}
```

Both `OS` and `ARCHITECTURE` are `const` — the compiler picks the
right value at build time. See [`system.md`](./system.md) for the
per-vendor name maps.

## 6. Logging macros

```rust,no_run
use lighty_core::{trace_debug, trace_info, trace_warn, trace_error};

fn process(path: &str) {
    trace_info!("processing {}", path);
    if path.is_empty() {
        trace_warn!("empty path");
        return;
    }
    trace_debug!("ok");
    let _ = path; trace_error!("never happens");
}
```

Without the `tracing` feature these expand to no-ops — keep them in
hot paths without runtime cost. See [`macros.md`](./macros.md).

## 7. Directory helpers

```rust,no_run
use lighty_core::{mkdir, join_and_mkdir, join_and_mkdir_vec};
use std::path::Path;

#[tokio::main]
async fn main() {
    let base = Path::new("/tmp/launcher");
    mkdir!(base);
    let mods = join_and_mkdir!(base, "instances/foo/mods");
    let world = join_and_mkdir_vec!(base, &["instances", "foo", "saves", "world1"]);
    println!("{} / {}", mods.display(), world.display());
}
```

## Errors at a glance

```rust,ignore
pub enum AppStateError   { AlreadyInitialized, NotInitialized, MissingPlatformDir(&'static str) }
pub enum DownloadError   { Http(reqwest::Error), Io(io::Error) }
pub enum ExtractError    { ZipEntryNotFound { .. }, AbsolutePath { .. }, PathTraversal { .. },
                           FileTooLarge { .. }, Zip(_), Tar(_), InvalidPath }
pub enum HashError       { Io(io::Error), Mismatch { expected, actual } }
pub enum SystemError     { UnsupportedOS, UnsupportedArchitecture, Io(io::Error) }
pub enum QueryError      { /* shared across the workspace, see exports.md */ }
```

## See also

- [`exports.md`](./exports.md) — full public surface
- [`events.md`](./events.md) — `CoreEvent` variants emitted by `extract`
- [`../../event/docs/events.md`](../../event/docs/events.md) — workspace event catalogue
