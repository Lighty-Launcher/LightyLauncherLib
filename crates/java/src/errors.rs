// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

use thiserror::Error;

/// Errors related to Java Runtime Environment (JRE) operations
#[derive(Debug, Error)]
pub enum JreError {
    #[error("JRE not found at {path:?}")]
    NotFound { path: std::path::PathBuf },

    #[error("Invalid JRE structure in directory")]
    InvalidStructure,

    #[error("Unsupported operating system for JRE installation")]
    UnsupportedOS,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Distribution error: {0}")]
    Distribution(#[from] DistributionError),

    #[error("Transfer failed: {0}")]
    Transfer(#[from] lighty_core::errors::DownloadError),

    #[error("Archive error: {0}")]
    Archive(#[from] lighty_core::errors::ExtractError),
}

/// Errors related to Java runtime execution
#[derive(Debug, Error)]
pub enum JavaRuntimeError {
    #[error("Java runtime not found at {path:?}")]
    NotFound { path: std::path::PathBuf },

    #[error("Process exited with non-zero exit code: {code}")]
    NonZeroExit { code: i32 },

    #[error("Failed to capture process I/O - stdout/stderr not configured")]
    IoCaptureFailure,

    #[error("Process spawn error: {0}")]
    Spawn(#[from] std::io::Error),

    #[error("Process terminated by signal")]
    SignalTerminated,
}

/// Errors related to Java distribution management
#[derive(Debug, Error)]
pub enum DistributionError {
    #[error("Unsupported Java version {version} for distribution {distribution}")]
    UnsupportedVersion { version: u8, distribution: &'static str },

    #[error("API error for {distribution}: {error}")]
    ApiError { distribution: &'static str, error: String },

    #[error("JSON parse error for {distribution}: {error}")]
    JsonParseError { distribution: &'static str, error: String },

    #[error("No packages found for {distribution}")]
    NoPackagesFound { distribution: &'static str },

    #[error("System error: {0}")]
    System(#[from] lighty_core::SystemError),
}

/// Type alias for JRE operations results
pub type JreResult<T> = Result<T, JreError>;

/// Type alias for Java runtime operations results
pub type JavaRuntimeResult<T> = Result<T, JavaRuntimeError>;

/// Type alias for Java distribution operations results
pub type DistributionResult<T> = Result<T, DistributionError>;
