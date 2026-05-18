# Exports

Public surface of `lighty-java`.

## Root re-exports

```rust
pub use lighty_java::{
    JavaDistribution, DistributionSelection,
    JreError,             JreResult,
    JavaRuntimeError,     JavaRuntimeResult,
    DistributionError,    DistributionResult,
};
```

## Module-by-module

### `jre_downloader`

```rust
pub async fn find_java_binary(
    runtimes_folder: &Path,
    distribution: &JavaDistribution,
    version: &u8,
) -> JreResult<PathBuf>;

#[cfg(feature = "events")]
pub async fn jre_download<F: Fn(u64, u64)>(
    runtimes_folder: &Path,
    distribution: &JavaDistribution,
    version: &u8,
    on_progress: F,
    event_bus: Option<&EventBus>,
) -> JreResult<PathBuf>;

#[cfg(not(feature = "events"))]
pub async fn jre_download<F: Fn(u64, u64)>(
    runtimes_folder: &Path,
    distribution: &JavaDistribution,
    version: &u8,
    on_progress: F,
) -> JreResult<PathBuf>;
```

### `runtime`

```rust
pub struct JavaRuntime(pub PathBuf);

impl JavaRuntime {
    pub fn new(path: PathBuf) -> Self;
    pub async fn execute(&self, arguments: Vec<String>, game_dir: &Path)
        -> JavaRuntimeResult<tokio::process::Child>;
    pub async fn handle_io<D: Send + Sync>(
        &self,
        process: &mut tokio::process::Child,
        on_stdout: fn(&D, &[u8]) -> JavaRuntimeResult<()>,
        on_stderr: fn(&D, &[u8]) -> JavaRuntimeResult<()>,
        terminator: tokio::sync::oneshot::Receiver<()>,
        data: &D,
    ) -> JavaRuntimeResult<()>;
}
```

### Top-level types

```rust
pub enum JavaDistribution { Temurin, GraalVM, Zulu, Liberica }

impl JavaDistribution {
    pub fn get_name(&self) -> &str;                              // "temurin" | "graalvm" | "zulu" | "liberica"
    pub fn supports_version(&self, version: u8) -> bool;
    pub fn get_fallback(&self, version: u8) -> Option<JavaDistribution>;
    pub async fn get_download_url(&self, jre_version: &u8) -> DistributionResult<String>;
}

pub enum DistributionSelection {
    Automatic(String),                  // serde tag = "automatic"
    Custom(String),                     // serde tag = "custom"
    Manual(JavaDistribution),           // serde tag = "manual"
}
```

`Default for JavaDistribution` returns `Temurin`. `Default for
DistributionSelection` returns `Automatic(String::new())`.

### Errors

```rust
pub enum JreError {
    NotFound { path: PathBuf }, InvalidStructure, Download(String),
    UnsupportedOS, Io(std::io::Error), Extraction(String),
}

pub enum JavaRuntimeError {
    NotFound { path: PathBuf }, NonZeroExit { code: i32 },
    IoCaptureFailure, Spawn(std::io::Error), SignalTerminated,
}

pub enum DistributionError {
    UnsupportedVersion { version: u8, distribution: &'static str },
    ApiError    { distribution: &'static str, error: String },
    JsonParseError { distribution: &'static str, error: String },
    NoPackagesFound { distribution: &'static str },
    System(lighty_core::SystemError),
}
```

## Cargo features

| Feature | Effect |
|---|---|
| `events` | `jre_download` gains the `Option<&EventBus>` arg and emits `JavaEvent`. |

## See also

- [`how-to-use.md`](./how-to-use.md) — install + run a JRE
- [`distributions.md`](./distributions.md) — provider matrix
- [`installation.md`](./installation.md) — `jre_download` pipeline
- [`runtime.md`](./runtime.md) — `JavaRuntime` API
- [`events.md`](./events.md) — `JavaEvent` variants
