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

    #[error("Loader not supported: {0}")]
    UnsupportedLoader(String),

    #[error("Invalid metadata: expected Version")]
    InvalidMetadata,

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Download failed: {0}")]
    DownloadFailed(String),

    #[error("Zip error: {0}")]
    Zip(#[from] async_zip::error::ZipError),

    #[error("Unable to get process ID from child process")]
    NoPid,
}

pub type InstallerResult<T> = std::result::Result<T, InstallerError>;
