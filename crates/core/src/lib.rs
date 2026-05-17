//! Foundation utilities (AppState, HTTP client, hashing, download, extract, OS probe).

pub mod system;
pub mod macros;
pub mod hosts;
pub mod download;
pub mod extract;
pub mod hash;
pub mod errors;
pub mod app_state;

pub use errors::{
    SystemError, SystemResult,
    ExtractError, ExtractResult,
    DownloadError, DownloadResult,
    AppStateError, AppStateResult,
    QueryError, QueryResult,
};

pub use hash::{
    HashError, HashResult,
    verify_file_sha1, verify_file_sha1_streaming,
    calculate_file_sha1_sync, verify_file_sha1_sync,
    calculate_sha1_bytes, calculate_sha1_bytes_raw,
};

pub use app_state::AppState;