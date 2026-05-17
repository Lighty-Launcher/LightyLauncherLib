// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Per-distribution download URL dispatch.

use crate::errors::DistributionResult;
use crate::JavaDistribution;

use super::providers;

/// Gets the download URL for `distribution` at `jre_version`.
pub(crate) async fn get_download_url(
    distribution: &JavaDistribution,
    jre_version: &u8,
) -> DistributionResult<String> {
    match distribution {
        JavaDistribution::Temurin => providers::build_temurin_url(jre_version),
        JavaDistribution::GraalVM => providers::build_graalvm_url(jre_version),
        JavaDistribution::Zulu => providers::build_zulu_url(jre_version).await,
        JavaDistribution::Liberica => providers::build_liberica_url(jre_version).await,
    }
}
