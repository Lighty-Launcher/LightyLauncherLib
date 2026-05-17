use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Event emitted when a game instance is launched.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceLaunchedEvent {
    pub pid: u32,
    pub instance_name: String,
    pub version: String,
    pub username: String,
    #[serde(with = "system_time_serializer")]
    pub timestamp: SystemTime,
}

/// Event emitted when a game instance window appears.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceWindowAppearedEvent {
    pub pid: u32,
    pub instance_name: String,
    pub version: String,
    #[serde(with = "system_time_serializer")]
    pub timestamp: SystemTime,
}

/// Event emitted when a game instance exits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceExitedEvent {
    pub pid: u32,
    pub instance_name: String,
    pub exit_code: Option<i32>,
    #[serde(with = "system_time_serializer")]
    pub timestamp: SystemTime,
}

/// Event emitted for each line of console output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleOutputEvent {
    pub pid: u32,
    pub instance_name: String,
    pub stream: ConsoleStream,
    pub line: String,
    #[serde(with = "system_time_serializer")]
    pub timestamp: SystemTime,
}

/// Event emitted when an instance is deleted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceDeletedEvent {
    pub instance_name: String,
    #[serde(with = "system_time_serializer")]
    pub timestamp: SystemTime,
}

/// Console stream type.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ConsoleStream {
    Stdout,
    Stderr,
}

mod system_time_serializer {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let duration = time
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        serializer.serialize_u64(duration.as_secs())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(UNIX_EPOCH + std::time::Duration::from_secs(secs))
    }
}
