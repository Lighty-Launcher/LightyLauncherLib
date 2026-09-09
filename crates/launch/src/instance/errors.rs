use thiserror::Error;

/// Result type for instance operations
pub type InstanceResult<T> = Result<T, InstanceError>;

/// Errors that can occur during instance management
#[derive(Debug, Error)]
pub enum InstanceError {
    #[error("Instance with PID {pid} not found")]
    NotFound { pid: u32 },

    #[error("Cannot delete instance '{instance_name}': still running with PIDs {pids:?}")]
    StillRunning {
        instance_name: String,
        pids: Vec<u32>,
    },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Cannot register instance: PID {pid} already tracked by '{existing_instance}'")]
    DuplicatePid {
        pid: u32,
        existing_instance: String,
    },
}
