# download — async file downloads

Two helpers built on the shared `hosts::HTTP_CLIENT`. No retry logic,
no SHA1 verification — pair with [`hash`](./hash.md) when you need
either.

## API

```rust
pub async fn download_file_untracked(
    url: &str,
    path: impl AsRef<Path>,
) -> DownloadResult<()>;

pub async fn download_file<F: Fn(u64, u64)>(
    url: &str,
    on_progress: F,
) -> DownloadResult<Vec<u8>>;
```

`download_file_untracked` writes directly to disk and discards the
response body. `download_file` streams the response chunk by chunk into
a `Vec<u8>`, calling `on_progress(current, total)` after each chunk.
`total` is `0` when the server omits `Content-Length`.

## Examples

### Save straight to disk

```rust
use lighty_core::download::download_file_untracked;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    download_file_untracked(
        "https://example.com/file.bin",
        "/tmp/file.bin",
    ).await?;
    Ok(())
}
```

### Buffer in memory with a progress callback

```rust
use lighty_core::download::download_file;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let bytes = download_file(
        "https://example.com/file.bin",
        |current, total| {
            if total > 0 {
                let pct = current as f64 * 100.0 / total as f64;
                println!("{:.1}%", pct);
            }
        },
    ).await?;
    tokio::fs::write("/tmp/file.bin", &bytes).await?;
    Ok(())
}
```

### Download + verify SHA1

```rust
use lighty_core::{download::download_file_untracked, hash::verify_file_sha1};
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let path = Path::new("/tmp/library.jar");
    download_file_untracked("https://example.com/library.jar", path).await?;

    if !verify_file_sha1(path, "expected-sha1").await? {
        tokio::fs::remove_file(path).await?;
        anyhow::bail!("library SHA1 mismatch");
    }
    Ok(())
}
```

### Concurrent downloads

```rust
use lighty_core::download::download_file_untracked;
use futures::future::try_join_all;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let jobs = vec![
        ("https://example.com/a", "/tmp/a"),
        ("https://example.com/b", "/tmp/b"),
        ("https://example.com/c", "/tmp/c"),
    ];
    try_join_all(jobs.into_iter().map(|(u, p)| download_file_untracked(u, p))).await?;
    Ok(())
}
```

## Errors

```rust
pub enum DownloadError {
    Http(reqwest::Error),   // network failure or non-2xx status
    Io(std::io::Error),     // write to disk failed
}
```

`error_for_status()` is called on the response, so 4xx/5xx surface as
`Http(_)`.

## See also

- [`hash.md`](./hash.md) — verify what you downloaded
- [`how-to-use.md`](./how-to-use.md) — top-level walkthrough
