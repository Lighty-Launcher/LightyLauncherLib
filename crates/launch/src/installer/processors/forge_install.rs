// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Forge/NeoForge install-processors wrappers.

use std::path::PathBuf;

use lighty_loaders::types::VersionInfo;
use lighty_core::QueryError;
use lighty_loaders::utils::forge_installer::ForgeInstallProfile;
use lighty_loaders::utils::maven::fetch_maven_sha1;

use super::processor::run_processors;
#[cfg(feature = "forge")]
use super::processor::extract_maven_bundle_to_libraries;

type Result<T> = std::result::Result<T, QueryError>;

/// Path to the per-installer marker file that records the SHA1 of the
/// installer whose processors last ran successfully.
fn processors_marker_path<V: VersionInfo>(version: &V, dot_dir: &str) -> PathBuf {
    version.game_dirs().join(dot_dir).join(format!(
        "processors-{}-{}.sha1",
        version.minecraft_version(),
        version.loader_version()
    ))
}

/// Runs the modern Forge install processors (>= 1.13).
///
/// Requires that the install_profile libraries are already on disk.
#[cfg(feature = "forge")]
pub(crate) async fn run_forge_install_processors<V: VersionInfo>(
    version: &V,
    install_profile: &ForgeInstallProfile,
    java_path: PathBuf,
) -> Result<()> {
    use lighty_loaders::forge::forge::{
        build_installer_url, installer_cache_path, FORGE_EXTRACT_SUBDIR, FORGE_MAVEN,
    };

    lighty_core::trace_info!(loader = "forge", "Checking if processors need to run");

    let installer_path = installer_cache_path(version);
    if !installer_path.exists() {
        return Err(QueryError::Conversion {
            message: "Installer JAR not found. Run fetch_full_data first.".to_string(),
        });
    }

    // Some Forge versions ship runtime artifacts at `/maven/...` inside the
    // installer (forge-shim.jar in 1.21+, etc.). Idempotent.
    let libraries_dir = version.game_dirs().join("libraries");
    extract_maven_bundle_to_libraries(&installer_path, &libraries_dir).map_err(|e| {
        QueryError::Conversion {
            message: format!(
                "Failed to extract /maven/ bundle from Forge installer {}: {e}",
                installer_path.display()
            ),
        }
    })?;

    let installer_url = build_installer_url(version);
    let marker_path = processors_marker_path(version, ".forge");
    if let Some(expected_sha1) = fetch_maven_sha1(&installer_url).await {
        if let Ok(existing) = std::fs::read_to_string(&marker_path) {
            if existing.trim() == expected_sha1 {
                lighty_core::trace_info!(
                    loader = "forge",
                    "Processors already executed for this installer, skipping"
                );
                return Ok(());
            }
        }
    }

    run_processors(
        version,
        install_profile,
        installer_path,
        FORGE_MAVEN,
        FORGE_EXTRACT_SUBDIR,
        java_path,
    )
    .await?;

    // Marker write failure must surface: silently swallowing would re-run
    // the heavy processors on the next launch.
    if let Some(expected_sha1) = fetch_maven_sha1(&installer_url).await {
        std::fs::write(&marker_path, expected_sha1).map_err(|e| QueryError::Conversion {
            message: format!(
                "Failed to write Forge processors marker {}: {e}",
                marker_path.display()
            ),
        })?;
    }

    lighty_core::trace_info!(loader = "forge", "Processors completed successfully");
    Ok(())
}

/// Runs the NeoForge install processors.
///
/// Requires that the install_profile libraries are already on disk.
#[cfg(feature = "neoforge")]
pub(crate) async fn run_neoforge_install_processors<V: VersionInfo>(
    version: &V,
    install_profile: &ForgeInstallProfile,
    java_path: PathBuf,
) -> Result<()> {
    use lighty_loaders::neoforge::neoforge::{
        build_installer_url, installer_cache_path, NEOFORGE_EXTRACT_SUBDIR, NEOFORGE_MAVEN,
    };

    lighty_core::trace_info!(loader = "neoforge", "Checking if processors need to run");

    let installer_path = installer_cache_path(version);
    if !installer_path.exists() {
        return Err(QueryError::Conversion {
            message: "Installer JAR not found. Run fetch_full_data first.".to_string(),
        });
    }

    let installer_url = build_installer_url(version);
    let marker_path = processors_marker_path(version, ".neoforge");
    if let Some(expected_sha1) = fetch_maven_sha1(&installer_url).await {
        if let Ok(existing) = std::fs::read_to_string(&marker_path) {
            if existing.trim() == expected_sha1 {
                lighty_core::trace_info!(
                    loader = "neoforge",
                    "Processors already executed for this installer, skipping"
                );
                return Ok(());
            }
        }
    }

    run_processors(
        version,
        install_profile,
        installer_path,
        NEOFORGE_MAVEN,
        NEOFORGE_EXTRACT_SUBDIR,
        java_path,
    )
    .await?;

    // Same as Forge: marker failure must surface, not be lost.
    if let Some(expected_sha1) = fetch_maven_sha1(&installer_url).await {
        std::fs::write(&marker_path, expected_sha1).map_err(|e| QueryError::Conversion {
            message: format!(
                "Failed to write NeoForge processors marker {}: {e}",
                marker_path.display()
            ),
        })?;
    }

    lighty_core::trace_info!(loader = "neoforge", "Processors completed successfully");
    Ok(())
}
