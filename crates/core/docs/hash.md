# hash — SHA1 helpers

Six functions covering the (sync vs async) × (file vs bytes) × (whole
file vs streaming) matrix. All hex comparisons are case-insensitive.

## API

```rust,ignore
// Async — files
pub async fn verify_file_sha1          (path: &Path, expected_sha1: &str) -> HashResult<bool>;
pub async fn verify_file_sha1_streaming(path: &Path, expected_sha1: &str) -> HashResult<bool>;

// Sync — files
pub fn calculate_file_sha1_sync(path: &Path)                            -> HashResult<String>;
pub fn verify_file_sha1_sync   (path: &Path, expected_sha1: &str)       -> HashResult<bool>;

// In-memory bytes
pub fn calculate_sha1_bytes    (data: &[u8]) -> String;     // 40-char hex
pub fn calculate_sha1_bytes_raw(data: &[u8]) -> [u8; 20];   // raw bytes
```

All four file helpers are also re-exported at the crate root
(`lighty_core::verify_file_sha1`, …).

## Which one to use

| File size | Function |
|---|---|
| < 100 MB | `verify_file_sha1` (reads the whole file into a `Vec<u8>`) |
| > 100 MB | `verify_file_sha1_streaming` (8 KiB buffer, constant memory) |
| Sync context (no Tokio runtime) | `*_sync` variants |
| Have raw bytes already | `calculate_sha1_bytes` / `calculate_sha1_bytes_raw` |

The streaming variant is roughly 5–10 % slower than `verify_file_sha1`
on small files; pick it for anything you expect to grow.

## Examples

### Verify a freshly downloaded library

```rust,no_run
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

### Stream a multi-GB JRE archive

```rust,no_run
use lighty_core::hash::verify_file_sha1_streaming;
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ok = verify_file_sha1_streaming(
        Path::new("/tmp/temurin-21.tar.gz"),
        "expected-sha1",
    ).await?;
    println!("ok = {ok}");
    Ok(())
}
```

### Hash an offline username

```rust,no_run
use lighty_core::hash::calculate_sha1_bytes;

fn main() {
    let hex = calculate_sha1_bytes(b"Player123");
    println!("{}", hex); // 40-char hex string
}
```

## Errors

```rust,ignore
pub enum HashError {
    Io(std::io::Error),
    Mismatch { expected: String, actual: String },   // not currently returned by these helpers,
                                                     // but reserved for higher-level wrappers
}
```

The `verify_*` helpers return `Ok(bool)` rather than `Err(Mismatch)` —
callers decide whether a mismatch is an error.

## See also

- [`download.md`](./download.md) — download then verify
- [`how-to-use.md`](./how-to-use.md) — top-level walkthrough
