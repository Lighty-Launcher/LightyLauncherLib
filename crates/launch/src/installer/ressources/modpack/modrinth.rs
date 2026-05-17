// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Modrinth `.mrpack`: manifest -> `Vec<Mods>` and overrides copy.

#![cfg(feature = "modrinth")]

use std::path::Path;

use lighty_core::QueryError;
use lighty_loaders::types::version_metadata::Mods;
use lighty_loaders::types::VersionInfo;
use lighty_modsloader::modrinth::modpack::parse_manifest;

use crate::errors::{InstallerError, InstallerResult};

use super::overrides::extract_overrides;

pub(super) async fn process_mrpack<V: VersionInfo>(
    work_dir: &Path,
    version: &V,
) -> InstallerResult<Vec<Mods>> {
    // Manifest parsing is sync I/O — run it on a blocking thread.
    let work_dir_owned = work_dir.to_path_buf();
    let manifest = tokio::task::spawn_blocking(move || parse_manifest(&work_dir_owned))
        .await
        .map_err(|e| InstallerError::Query(QueryError::Conversion {
            message: format!("Manifest parse task panicked: {e}"),
        }))??;

    // Reconcile loader + MC: warn-only. Builder's values still win downstream.
    if version.minecraft_version() != manifest.dependencies.minecraft {
        lighty_core::trace_warn!(
            "[Modpack] Manifest declares Minecraft {} but builder has {} — using builder's value",
            manifest.dependencies.minecraft,
            version.minecraft_version()
        );
    }

    let mut out = Vec::with_capacity(manifest.files.len());
    for file in manifest.files.iter().filter(|f| f.is_client_required()) {
        let url = file.downloads.first().ok_or_else(|| {
            InstallerError::Query(QueryError::MissingField {
                field: format!("downloads[0] for {}", file.path),
            })
        })?;
        out.push(Mods {
            name: file.path.clone(),
            url: Some(url.clone()),
            path: Some(file.path.clone()),
            sha1: Some(file.hashes.sha1.clone()),
            size: Some(file.file_size),
        });
    }

    let runtime = version.runtime_dir().to_path_buf();
    let extracted = extract_overrides(&work_dir.join("overrides"), &runtime).await?;
    let extracted_client = if work_dir.join("client-overrides").exists() {
        extract_overrides(&work_dir.join("client-overrides"), &runtime).await?
    } else {
        0
    };
    lighty_core::trace_info!(
        "[Modpack] Extracted overrides ({}) + client-overrides ({}) into runtime",
        extracted,
        extracted_client
    );

    Ok(out)
}
