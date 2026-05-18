# AppState — global launcher paths

`AppState` resolves and caches the per-launcher data / config / cache
directories at startup. Every other crate in the workspace reads from
the global accessors instead of re-deriving paths.

This file is the **canonical** reference for `AppState::init` and the
resulting OS paths. Other docs cross-link here.

## Initialise once

```rust,no_run
use lighty_core::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("LightyLauncher")?;

    println!("data:   {}", AppState::data_dir().display());
    println!("config: {}", AppState::config_dir().display());
    println!("cache:  {}", AppState::cache_dir().display());
    Ok(())
}
```

`init` is meant to be called **once** at process start. A second call
returns `AppStateError::AlreadyInitialized` and leaves the original
paths in place. The accessors below panic with a clear message if you
forget to initialise — it's a programmer error, not a runtime one.

## Resulting paths

The launcher name passed to `init` becomes a sub-directory under the
OS-standard bases (resolved via the `dirs` crate). With
`AppState::init("LightyLauncher")` you get:

| OS      | `data_dir()`                                    | `config_dir()`                                  | `cache_dir()`                       |
|---------|-------------------------------------------------|-------------------------------------------------|-------------------------------------|
| Linux   | `~/.local/share/LightyLauncher/`                | `~/.config/LightyLauncher/`                     | `~/.cache/LightyLauncher/`          |
| macOS   | `~/Library/Application Support/LightyLauncher/` | `~/Library/Application Support/LightyLauncher/` | `~/Library/Caches/LightyLauncher/`  |
| Windows | `%APPDATA%\LightyLauncher\`                     | `%APPDATA%\LightyLauncher\`                     | `%LOCALAPPDATA%\LightyLauncher\`    |

On Linux/macOS the launcher name is **not** prefixed with a dot — the
directory is visible. Windows uses `APPDATA` for both `data` and
`config` because the platform doesn't separate the two.

## API surface

```rust,ignore
pub struct AppState;

impl AppState {
    pub fn init(name: impl Into<String>) -> AppStateResult<()>;

    pub fn paths()       -> &'static LauncherPaths;
    pub fn name()        -> &'static str;
    pub fn data_dir()    -> &'static Path;
    pub fn config_dir()  -> &'static Path;
    pub fn cache_dir()   -> &'static Path;
    pub fn app_version() -> &'static str;     // env!("CARGO_PKG_VERSION")
    pub fn client_id()   -> &'static str;     // RFC 4122 v4, persisted
}

pub struct LauncherPaths {
    pub name:       String,
    pub data_dir:   PathBuf,
    pub config_dir: PathBuf,
    pub cache_dir:  PathBuf,
}
```

`client_id()` is the per-install launcher id surfaced as `${clientid}`
in the JVM args. It's persisted at `<config_dir>/client_id` so crash
reports and Mojang telemetry stay correlated across sessions. First
call mints a fresh UUID v4 and writes it back; subsequent calls in the
same process hit an in-memory `OnceCell`.

## What goes where

Conventions used across the workspace:

| Directory | Lives under  | Holds                                                                |
|-----------|--------------|----------------------------------------------------------------------|
| `data`    | `data_dir()` | Instances (`instances/<name>/`), shared `libraries/`, `assets/`      |
| `config`  | `config_dir()` | `client_id`, user-editable settings                                |
| `cache`   | `cache_dir()` | Bundled JREs (`java/<distribution>_<version>/`), modpack archive cache |

The launch crate writes Forge / NeoForge installer caches under each
instance's `.forge/` directory inside `data_dir()`, not in `cache_dir()`.

## Thread safety

Paths are stored in a `OnceCell`, accessors return `&'static`
references — safe to call from any thread or async task once `init`
has succeeded.

## Errors

```rust,ignore
pub enum AppStateError {
    AlreadyInitialized,
    NotInitialized,
    MissingPlatformDir(&'static str),   // "data" | "config" | "cache"
}
```

`MissingPlatformDir` fires only on exotic targets where the `dirs`
crate can't resolve a base directory (no `$HOME`, no `%APPDATA%`, …).

## See also

- [`overview.md`](./overview.md) — what `lighty-core` is for
- [`how-to-use.md`](./how-to-use.md) — full crate walkthrough
- [`../../version/docs/how-to-use.md`](../../version/docs/how-to-use.md)
  — `VersionBuilder` reads `AppState` for its default paths
