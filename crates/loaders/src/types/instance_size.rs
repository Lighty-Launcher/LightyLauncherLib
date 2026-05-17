use serde::{Deserialize, Serialize};

/// Disk-size breakdown of a Minecraft instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceSize {
    pub libraries: u64,
    pub mods: u64,
    pub natives: u64,
    pub client: u64,
    pub assets: u64,
    pub total: u64,
}

impl InstanceSize {
    /// Get total size in megabytes.
    pub fn total_mb(&self) -> f64 {
        self.total as f64 / 1024.0 / 1024.0
    }

    /// Get total size in gigabytes.
    pub fn total_gb(&self) -> f64 {
        self.total as f64 / 1024.0 / 1024.0 / 1024.0
    }

    /// Format bytes as a string with appropriate unit (B, KB, MB, GB).
    pub fn format(bytes: u64) -> String {
        if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.2} KB", bytes as f64 / 1024.0)
        } else if bytes < 1024 * 1024 * 1024 {
            format!("{:.2} MB", bytes as f64 / 1024.0 / 1024.0)
        } else {
            format!("{:.2} GB", bytes as f64 / 1024.0 / 1024.0 / 1024.0)
        }
    }
}
