# Exports

Public surface of `lighty-core`.

## Root re-exports

```rust,ignore
pub use lighty_core::{
    AppState,
    // Errors
    AppStateError, AppStateResult,
    DownloadError, DownloadResult,
    ExtractError,  ExtractResult,
    HashError,     HashResult,
    SystemError,   SystemResult,
    QueryError,    QueryResult,
    // SHA1 (re-exported flat for convenience)
    verify_file_sha1, verify_file_sha1_streaming,
    calculate_file_sha1_sync, verify_file_sha1_sync,
    calculate_sha1_bytes, calculate_sha1_bytes_raw,
};
```

## Module-by-module

### `app_state`

```rust,ignore
pub struct AppState;
pub struct LauncherPaths { pub name: String, pub data_dir: PathBuf,
                           pub config_dir: PathBuf, pub cache_dir: PathBuf }
```

Methods: `init`, `paths`, `name`, `data_dir`, `config_dir`,
`cache_dir`, `app_version`, `client_id`. Detail in
[`app_state.md`](./app_state.md).

### `download`

```rust,ignore
pub async fn download_file_untracked(url: &str, path: impl AsRef<Path>) -> DownloadResult<()>;
pub async fn download_file<F: Fn(u64, u64)>(url: &str, on_progress: F) -> DownloadResult<Vec<u8>>;
```

### `extract`

```rust,ignore
// Without `events`
pub async fn zip_extract<R>(archive: R, out_dir: &Path) -> ExtractResult<()>
where R: AsyncRead + AsyncSeek + Unpin + AsyncBufRead;

pub async fn tar_gz_extract<R>(archive: R, out_dir: &Path) -> ExtractResult<()>
where R: AsyncRead + Unpin;

// With `events`
pub async fn zip_extract<R>(archive: R, out_dir: &Path,
                            event_bus: Option<&EventBus>) -> ExtractResult<()>;
pub async fn tar_gz_extract<R>(archive: R, out_dir: &Path,
                               event_bus: Option<&EventBus>) -> ExtractResult<()>;
```

### `hash`

```rust,ignore
pub async fn verify_file_sha1          (path: &Path, expected_sha1: &str) -> HashResult<bool>;
pub async fn verify_file_sha1_streaming(path: &Path, expected_sha1: &str) -> HashResult<bool>;
pub fn       calculate_file_sha1_sync  (path: &Path) -> HashResult<String>;
pub fn       verify_file_sha1_sync     (path: &Path, expected_sha1: &str) -> HashResult<bool>;
pub fn       calculate_sha1_bytes      (data: &[u8]) -> String;       // 40-char hex
pub fn       calculate_sha1_bytes_raw  (data: &[u8]) -> [u8; 20];     // raw bytes
```

### `system`

```rust,ignore
pub const OS:           OperatingSystem;
pub const ARCHITECTURE: Architecture;

pub enum OperatingSystem { WINDOWS, LINUX, OSX, UNKNOWN }
pub enum Architecture    { X86, X64, ARM, AARCH64, UNKNOWN }
```

OS methods: `get_vanilla_os`, `get_adoptium_name`, `get_graal_name`,
`get_zulu_name`, `get_zulu_ext`, `get_archive_type`.

Arch methods: `get_simple_name`, `get_vanilla_arch`, `get_arch_bits`,
`get_zulu_arch`.

### `hosts`

```rust,ignore
pub static HTTP_CLIENT: Lazy<reqwest::Client>;
```

Single shared client — use this instead of building your own.

### `macros` (auto-imported from the crate root)

```rust,ignore
mkdir!($path)                       // async create_dir_all, swallows errors
try_mkdir!($path)                   // async create_dir_all, returns io::Result
mkdir_blocking!($path)              // spawn_blocking variant
join_and_mkdir!($path, $join)       // join + mkdir, returns PathBuf
join_and_mkdir_vec!($path, $joins)  // join sequence + mkdir, returns PathBuf

trace_debug!(...)  trace_info!(...) trace_warn!(...) trace_error!(...)
time_it!($label, $expr)             // debug-logs elapsed Duration
```

The `trace_*!` and `time_it!` macros expand to no-ops without the
`tracing` feature.

### `errors`

All error / result aliases live here. They're re-exported flat at the
crate root (see [Root re-exports](#root-re-exports)).

## Cargo features

```toml
[dependencies]
lighty-core = { version = "...", features = ["events", "tracing"] }
```

| Feature   | Effect                                                                   |
|-----------|--------------------------------------------------------------------------|
| `events`  | Adds the `Option<&EventBus>` arg to `zip_extract` / `tar_gz_extract`.    |
| `tracing` | Routes the `trace_*!` macros to `tracing::*` (otherwise no-ops).         |

## See also

- [`how-to-use.md`](./how-to-use.md) — one example per module
- [`events.md`](./events.md) — the three `CoreEvent` variants
