//! Process-wide launcher paths resolved via the `dirs` crate.

use std::path::{Path, PathBuf};

use once_cell::sync::OnceCell;

use crate::errors::{AppStateError, AppStateResult};

const CLIENT_ID_FILE: &str = "client_id";

/// Resolved per-launcher paths.
#[derive(Debug, Clone)]
pub struct LauncherPaths {
    pub name: String,
    pub data_dir: PathBuf,
    pub config_dir: PathBuf,
    pub cache_dir: PathBuf,
}

static PATHS: OnceCell<LauncherPaths> = OnceCell::new();
static CLIENT_ID: OnceCell<String> = OnceCell::new();

/// Zero-sized handle for the global launcher paths.
pub struct AppState;

impl AppState {
    /// Initialises the global launcher paths. Call once at startup.
    ///
    /// `name` becomes the per-launcher subdirectory under the OS-
    /// standard data/config/cache bases. Returns
    /// [`AppStateError::AlreadyInitialized`] on a second call.
    pub fn init(name: impl Into<String>) -> AppStateResult<()> {
        let name = name.into();
        let data_dir = dirs::data_dir()
            .ok_or(AppStateError::MissingPlatformDir("data"))?
            .join(&name);
        let config_dir = dirs::config_dir()
            .ok_or(AppStateError::MissingPlatformDir("config"))?
            .join(&name);
        let cache_dir = dirs::cache_dir()
            .ok_or(AppStateError::MissingPlatformDir("cache"))?
            .join(&name);
        PATHS
            .set(LauncherPaths { name, data_dir, config_dir, cache_dir })
            .map_err(|_| AppStateError::AlreadyInitialized)
    }

    /// Returns the resolved launcher paths.
    ///
    /// Panics with a clear message if [`Self::init`] hasn't been
    /// called — that's a programmer error, not a runtime condition.
    pub fn paths() -> &'static LauncherPaths {
        PATHS.get().expect(
            "AppState::init(\"<launcher-name>\") must be called once at startup",
        )
    }

    /// Launcher name as supplied to [`Self::init`].
    pub fn name() -> &'static str {
        &Self::paths().name
    }

    /// Persistent data directory (instances live here).
    pub fn data_dir() -> &'static Path {
        &Self::paths().data_dir
    }

    /// User configuration directory (the bundled JRE lives here).
    pub fn config_dir() -> &'static Path {
        &Self::paths().config_dir
    }

    /// Disposable cache directory.
    pub fn cache_dir() -> &'static Path {
        &Self::paths().cache_dir
    }

    /// Application version derived from `CARGO_PKG_VERSION`.
    pub fn app_version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    /// Per-install launcher client id, surfaced to the JVM as `${clientid}`.
    ///
    /// Persisted at `<config_dir>/client_id` so crash reports and Mojang
    /// telemetry stay correlated across sessions. On first call we read it
    /// from disk; on a missing/unreadable file we mint a fresh UUID v4
    /// (RFC 4122) and write it back. Subsequent calls in the same process
    /// hit the in-memory cache.
    pub fn client_id() -> &'static str {
        CLIENT_ID.get_or_init(|| {
            let path = Self::config_dir().join(CLIENT_ID_FILE);

            // Trim avoids trailing \n from manual edits or POSIX conventions.
            if let Ok(raw) = std::fs::read_to_string(&path) {
                let trimmed = raw.trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }

            let fresh = generate_uuid_v4();

            // Best-effort write: if config dir is unwritable we still return
            // a valid id so launches go through; next run will regenerate.
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Err(e) = std::fs::write(&path, &fresh) {
                crate::trace_debug!(
                    error = %e,
                    path = %path.display(),
                    "Could not persist client_id; continuing with in-memory value"
                );
            }

            fresh
        })
    }
}

/// Generates a RFC 4122 v4 UUID string from `fastrand`.
fn generate_uuid_v4() -> String {
    let mut bytes = [0u8; 16];
    for b in bytes.iter_mut() {
        *b = fastrand::u8(..);
    }
    // Version 4 (random): high nibble of byte 6 = 0b0100
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    // Variant RFC 4122: top two bits of byte 8 = 0b10
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5],
        bytes[6], bytes[7],
        bytes[8], bytes[9],
        bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
    )
}
