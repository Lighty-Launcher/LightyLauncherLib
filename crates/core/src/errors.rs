use thiserror::Error;

/// Errors related to operating system and architecture detection
#[derive(Debug, Error)]
pub enum SystemError {
    #[error("Unsupported operating system")]
    UnsupportedOS,

    #[error("Unsupported architecture")]
    UnsupportedArchitecture,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Errors related to file extraction (ZIP and tar.gz)
#[derive(Debug, Error)]
pub enum ExtractError {
    #[error("Invalid ZIP entry at index {index}")]
    ZipEntryNotFound { index: usize },

    #[error("Path has no parent directory")]
    InvalidPath,

    #[error("Absolute paths not allowed: {path}")]
    AbsolutePath { path: String },

    #[error("Path traversal detected: {path} escapes base directory")]
    PathTraversal { path: String },

    #[error("File too large: {size} bytes (max: {max})")]
    FileTooLarge { size: u64, max: u64 },

    #[error("ZIP error: {0}")]
    Zip(#[from] async_zip::error::ZipError),

    #[error("TAR error: {0}")]
    Tar(#[from] std::io::Error),
}

/// Errors related to HTTP downloads
#[derive(Debug, Error)]
pub enum DownloadError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Type alias for System operations results
pub type SystemResult<T> = Result<T, SystemError>;

/// Type alias for extraction operations results
pub type ExtractResult<T> = Result<T, ExtractError>;

/// Errors related to application state initialization
#[derive(Debug, Error)]
pub enum AppStateError {
    #[error("AppState::init() has already been called")]
    AlreadyInitialized,

    #[error("AppState::init() has not been called yet — call it once at startup")]
    NotInitialized,

    #[error("Platform doesn't expose a standard {0} directory")]
    MissingPlatformDir(&'static str),
}

/// Type alias for download operations results
pub type DownloadResult<T> = Result<T, DownloadError>;

/// Type alias for app state operations results
pub type AppStateResult<T> = Result<T, AppStateError>;

/// Errors returned by every loader / mods query operation.
#[derive(Error, Debug)]
pub enum QueryError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("JSON parsing error: {0}")]
    JsonParsing(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Version '{version}' not found in manifest")]
    VersionNotFound { version: String },

    #[error("Missing field '{field}' in manifest data")]
    MissingField { field: String },

    #[error("Assets fetch error: failed to fetch assets from URL '{url}'")]
    AssetsFetch { url: String },

    #[error("Conversion error: {message}")]
    Conversion { message: String },

    #[error("Archive corrupted ({context}): {reason}")]
    ArchiveCorrupted { context: String, reason: String },

    #[error("Unsupported {what}: expected {expected}, found {found}")]
    UnsupportedFormat {
        what: String,
        expected: String,
        found: String,
    },

    #[error("Unsupported loader: {0}")]
    UnsupportedLoader(String),

    #[error("Invalid metadata format")]
    InvalidMetadata,

    #[error("Mod not found on {provider}: {id}")]
    ModNotFound { provider: &'static str, id: String },

    #[error("Mod {id} on {provider} has no version compatible with Minecraft {mc} + loader {loader}")]
    ModIncompatible {
        provider: &'static str,
        id: String,
        mc: String,
        loader: String,
    },

    #[error("Mod {id} on CurseForge disallows third-party distribution; download manually from the project page")]
    ModDistributionForbidden { id: String },
}

/// Type alias for query operations results.
pub type QueryResult<T> = Result<T, QueryError>;
