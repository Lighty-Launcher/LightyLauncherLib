// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Global launch configuration shared across all launches.

use lighty_java::JavaDistribution;

/// Launch configuration shared across launches.
#[derive(Debug, Clone)]
pub struct LaunchConfig {
    pub username: String,
    pub uuid: String,
    pub java_distribution: JavaDistribution,
}

impl LaunchConfig {
    /// Create a new launch configuration.
    pub fn new(
        username: impl Into<String>,
        uuid: impl Into<String>,
        java_distribution: JavaDistribution,
    ) -> Self {
        Self {
            username: username.into(),
            uuid: uuid.into(),
            java_distribution,
        }
    }
}

impl Default for LaunchConfig {
    fn default() -> Self {
        Self {
            username: "Hamadi".to_string(),
            uuid: "00000000-0000-0000-0000-000000000000".to_string(),
            java_distribution: JavaDistribution::Temurin,
        }
    }
}

