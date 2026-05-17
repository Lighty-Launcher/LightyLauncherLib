// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Lighty Java - JRE download, install, and process execution.

mod distribution;
pub mod jre_downloader;
pub mod runtime;
pub mod errors;

use serde::{Deserialize, Serialize};

pub use errors::{
    JreError, JreResult,
    JavaRuntimeError, JavaRuntimeResult,
    DistributionError, DistributionResult,
};

/// Selection method for Java distribution
#[derive(Deserialize, Serialize, Clone)]
#[serde(tag = "type", content = "value")]
pub enum DistributionSelection {
    #[serde(rename = "automatic")]
    Automatic(String),
    #[serde(rename = "custom")]
    Custom(String),
    #[serde(rename = "manual")]
    Manual(JavaDistribution),
}

impl Default for DistributionSelection {
    fn default() -> Self {
        DistributionSelection::Automatic(String::new())
    }
}

/// Available Java distributions
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum JavaDistribution {
    #[serde(rename = "temurin")]
    Temurin,
    #[serde(rename = "graalvm")]
    GraalVM,
    #[serde(rename = "zulu")]
    Zulu,
    #[serde(rename = "liberica")]
    Liberica,
}

impl Default for JavaDistribution {
    fn default() -> Self {
        // Temurin is the default as it supports all Java versions
        JavaDistribution::Temurin
    }
}

impl JavaDistribution {
    /// Returns the canonical name of this distribution
    pub fn get_name(&self) -> &str {
        match self {
            JavaDistribution::Temurin => "temurin",
            JavaDistribution::GraalVM => "graalvm",
            JavaDistribution::Zulu => "zulu",
            JavaDistribution::Liberica => "liberica",
        }
    }

    /// Returns whether this distribution publishes `version` for the
    /// current platform.
    ///
    /// Known gaps:
    /// - Temurin: no Java 8 for macOS ARM64 (Apple Silicon released after
    ///   Java 8 reached EOL).
    /// - GraalVM: Java 17+ only.
    pub fn supports_version(&self, version: u8) -> bool {
        use lighty_core::system::{Architecture, OperatingSystem, ARCHITECTURE, OS};

        match self {
            JavaDistribution::Temurin => {
                // No Java 8 for macOS ARM64
                !(version == 8 && OS == OperatingSystem::OSX && ARCHITECTURE == Architecture::AARCH64)
            }
            JavaDistribution::GraalVM => version >= 17,
            JavaDistribution::Zulu | JavaDistribution::Liberica => true,
        }
    }

    /// Returns a fallback distribution for `(self, version)` if `self`
    /// does not support that version on the current platform.
    ///
    /// Returns `None` when no fallback is needed (`self` is supported).
    /// Fallback candidates are tried in order: Zulu → Liberica → Temurin
    /// (decreasing platform coverage).
    pub fn get_fallback(&self, version: u8) -> Option<JavaDistribution> {
        if self.supports_version(version) {
            return None;
        }

        // Zulu/Liberica/Temurin in decreasing platform coverage order.
        let candidates = [
            JavaDistribution::Zulu,
            JavaDistribution::Liberica,
            JavaDistribution::Temurin,
        ];

        candidates.into_iter().find(|d| d.supports_version(version))
    }

    /// Gets download URL for the distribution
    pub async fn get_download_url(&self, jre_version: &u8) -> DistributionResult<String> {
        distribution::get_download_url(self, jre_version).await
    }
}