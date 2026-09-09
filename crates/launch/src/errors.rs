use thiserror::Error;

/// Errors raised by the installer pipeline.
#[derive(Debug, Error)]
pub enum InstallerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("SHA1 verification failed: {0}")]
    Sha1(#[from] lighty_core::HashError),

    #[error("Query error: {0}")]
    Query(#[from] lighty_core::QueryError),

    #[error("Invalid metadata: expected Version")]
    InvalidMetadata,

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("HTTP {status} for {url}")]
    HttpStatus { status: u16, url: String },

    #[error("SHA1 mismatch for {url}: expected {expected}, got {actual}")]
    Sha1Mismatch {
        url: String,
        expected: String,
        actual: String,
    },

    #[error("Partial file for {url} is longer than the file itself")]
    StalePartialFile { url: String },

    #[error("Download of {url} failed after {attempts} attempts")]
    RetriesExhausted { attempts: u32, url: String },

    #[error("Download concurrency semaphore closed")]
    ConcurrencyClosed,

    #[error("Zip error: {0}")]
    Zip(#[from] async_zip::error::ZipError),

    #[error("JRE download failed: {0}")]
    Jre(#[from] lighty_java::JreError),

    #[error("Launch failed: {0}")]
    JavaRuntime(#[from] lighty_java::JavaRuntimeError),

    #[error("Modpack archive download failed: {0}")]
    ModpackArchive(#[from] lighty_core::errors::DownloadError),

    #[error("Unable to get process ID from child process")]
    NoPid,
}

pub type InstallerResult<T> = std::result::Result<T, InstallerError>;
