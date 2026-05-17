// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Oracle GraalVM distribution provider. JDK-only, Java 17+.

use crate::errors::{DistributionError, DistributionResult};
use lighty_core::system::{ARCHITECTURE, OS};

/// Builds GraalVM download URL. Supports Java 17+ only.
pub fn build_graalvm_url(version: &u8) -> DistributionResult<String> {
    let os_name = OS.get_graal_name()?;
    let arch = ARCHITECTURE.get_simple_name()?;
    let archive_type = OS.get_archive_type()?;

    let url = if *version > 17 {
        format!(
            "https://download.oracle.com/graalvm/{}/latest/graalvm-jdk-{}_{}-{}_bin.{}",
            version, version, os_name, arch, archive_type
        )
    } else if *version == 17 {
        format!(
            "https://download.oracle.com/graalvm/17/archive/graalvm-jdk-17.0.12_{}-{}_bin.{}",
            os_name, arch, archive_type
        )
    } else {
        return Err(DistributionError::UnsupportedVersion {
            version: *version,
            distribution: "GraalVM",
        });
    };

    Ok(url)
}
