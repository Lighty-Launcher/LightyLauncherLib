// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! CurseForge `.zip` modpack: manifest -> `Vec<Mods>` and overrides copy.

#![cfg(feature = "curseforge")]

use std::path::Path;

use futures::future::try_join_all;
use lighty_core::QueryError;
use lighty_loaders::types::version_metadata::Mods;
use lighty_loaders::types::VersionInfo;
use lighty_modsloader::curseforge::client::{fetch_pinned_file, install_subdir_for};
use lighty_modsloader::curseforge::client_metadata::HASH_ALGO_SHA1;
use lighty_modsloader::curseforge::modpack::parse_manifest;

use crate::errors::{InstallerError, InstallerResult};

use super::overrides::extract_overrides;

pub(super) async fn process_cf_modpack<V: VersionInfo>(
    work_dir: &Path,
    version: &V,
) -> InstallerResult<Vec<Mods>> {
    let work_dir_owned = work_dir.to_path_buf();
    let manifest = tokio::task::spawn_blocking(move || parse_manifest(&work_dir_owned))
        .await
        .map_err(|e| InstallerError::Query(QueryError::Conversion {
            message: format!("CF manifest parse task panicked: {e}"),
        }))??;

    if version.minecraft_version() != manifest.minecraft.version {
        lighty_core::trace_warn!(
            "[Modpack] CF manifest declares Minecraft {} but builder has {} — using builder's value",
            manifest.minecraft.version,
            version.minecraft_version()
        );
    }

    // Resolve each (projectID, fileID) concurrently via try_join_all so an
    // 80-mod pack hits the API in parallel instead of serially. Each file
    // also needs its parent project's classId to route it to the right
    // runtime subfolder (mods/resourcepacks/shaderpacks).
    let ttl = version.ttl();
    let out: Vec<Mods> = try_join_all(
        manifest
            .files
            .iter()
            .filter(|f| f.required)
            .map(|cf_file| async move {
                let (resolved, subdir) = tokio::try_join!(
                    fetch_pinned_file(cf_file.project_id, cf_file.file_id),
                    install_subdir_for(cf_file.project_id, ttl),
                )?;
                let url = resolved.download_url.ok_or_else(|| {
                    InstallerError::Query(QueryError::ModDistributionForbidden {
                        id: format!("{}/{}", cf_file.project_id, cf_file.file_id),
                    })
                })?;
                let sha1 = resolved
                    .hashes
                    .iter()
                    .find(|h| h.algo == HASH_ALGO_SHA1)
                    .map(|h| h.value.clone());
                Ok::<Mods, InstallerError>(Mods {
                    name: resolved.file_name.clone(),
                    url: Some(url),
                    path: Some(format!("{}/{}", subdir, resolved.file_name)),
                    sha1,
                    size: (resolved.file_length > 0).then_some(resolved.file_length),
                })
            }),
    )
    .await?;

    // The overrides directory name is configurable per-pack.
    let runtime = version.runtime_dir().to_path_buf();
    let overrides_dir = work_dir.join(&manifest.overrides);
    let extracted = if overrides_dir.exists() {
        extract_overrides(&overrides_dir, &runtime).await?
    } else {
        0
    };
    lighty_core::trace_info!(
        "[Modpack] CF: extracted {} overrides into runtime",
        extracted
    );

    Ok(out)
}
