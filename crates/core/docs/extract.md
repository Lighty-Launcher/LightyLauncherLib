# extract — ZIP and tar.gz extraction

Streaming archive extraction with hard caps on file size and strict
path validation. Supported formats: **ZIP** and **TAR.GZ**.

## API

```rust,ignore
// Without `events`
pub async fn zip_extract<R>(archive: R, out_dir: &Path) -> ExtractResult<()>
where R: AsyncRead + AsyncSeek + Unpin + AsyncBufRead;

pub async fn tar_gz_extract<R>(archive: R, out_dir: &Path) -> ExtractResult<()>
where R: AsyncRead + Unpin;

// With `events` (extra Option<&EventBus>)
pub async fn zip_extract<R>(archive: R, out_dir: &Path,
                            event_bus: Option<&EventBus>) -> ExtractResult<()>;
pub async fn tar_gz_extract<R>(archive: R, out_dir: &Path,
                               event_bus: Option<&EventBus>) -> ExtractResult<()>;
```

`out_dir` must already exist (the helpers call `canonicalize`). Pass a
`tokio::io::BufReader<tokio::fs::File>` or any reader satisfying the
trait bounds.

## Hard limits

- **2 GiB per entry** — defends against zip-bomb attacks. Larger
  entries raise `ExtractError::FileTooLarge`.
- **Path traversal rejected** — entries whose normalized destination
  escapes `out_dir` raise `ExtractError::PathTraversal`.
- **Absolute paths rejected** (zip only) — `ExtractError::AbsolutePath`.
- **Symlinks / hardlinks skipped** in tar.gz — silently ignored.

The 256 KiB buffer keeps memory usage flat regardless of input size.

## Example: zip an instance's `mods/`

```rust,no_run
use lighty_core::extract::zip_extract;
use tokio::{fs::File, io::BufReader};
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let archive = BufReader::new(File::open("mods.zip").await?);

    #[cfg(feature = "events")]
    zip_extract(archive, Path::new("./instance/mods"), None).await?;

    #[cfg(not(feature = "events"))]
    zip_extract(archive, Path::new("./instance/mods")).await?;
    Ok(())
}
```

## Example: extract a JRE tarball

```rust,no_run
use lighty_core::extract::tar_gz_extract;
use tokio::{fs::File, io::BufReader};
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let archive = BufReader::new(File::open("jre.tar.gz").await?);

    #[cfg(feature = "events")]
    tar_gz_extract(archive, Path::new("./runtimes/temurin_21"), None).await?;

    #[cfg(not(feature = "events"))]
    tar_gz_extract(archive, Path::new("./runtimes/temurin_21")).await?;
    Ok(())
}
```

## Errors

```rust,ignore
pub enum ExtractError {
    ZipEntryNotFound { index: usize },
    InvalidPath,
    AbsolutePath { path: String },
    PathTraversal { path: String },
    FileTooLarge { size: u64, max: u64 },
    Zip(async_zip::error::ZipError),
    Tar(std::io::Error),                 // shared with TAR + plain IO
}
```

## Events

With the `events` feature, the helpers emit `CoreEvent::Extraction*` on
the bus you pass. Detail in [`events.md`](./events.md).

## See also

- [`download.md`](./download.md) — pair with `download_file_untracked`
- [`how-to-use.md`](./how-to-use.md) — full crate walkthrough
- [`../../java/docs/installation.md`](../../java/docs/installation.md)
  — `lighty-java` uses these to install JREs
