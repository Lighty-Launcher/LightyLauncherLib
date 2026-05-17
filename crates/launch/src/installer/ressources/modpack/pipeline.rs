// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Modpack pipeline orchestrator: download, cache, extract, dispatch, mark installed.

#![cfg(any(feature = "modrinth", feature = "curseforge"))]

use std::path::Path;

use lighty_core::download::download_file_untracked;
use lighty_core::hash::calculate_sha1_bytes;
use lighty_core::{AppState, QueryError};
use lighty_loaders::types::version_metadata::Mods;
use lighty_loaders::types::VersionInfo;
use lighty_modsloader::modpack::ModpackSource;

use crate::errors::{InstallerError, InstallerResult};

#[cfg(feature = "events")]
use lighty_event::{Event, EventBus, ModloaderEvent};

/// Runs the modpack pipeline for `source` and returns the `Vec<Mods>`
/// to merge into the version pivot before install.
///
/// Returns `Ok(vec![])` when the marker already matches (idempotent
/// re-run — nothing to do).
pub(crate) async fn process<V>(
    source: &ModpackSource,
    version: &V,
    #[cfg(feature = "events")] event_bus: Option<&EventBus>,
) -> InstallerResult<Vec<Mods>>
where
    V: VersionInfo,
{
    let source_desc = describe_source(source);

    #[cfg(feature = "events")]
    if let Some(bus) = event_bus {
        bus.emit(Event::Modloader(ModloaderEvent::ModpackResolveStart {
            source: source_desc.clone(),
        }));
    }
    lighty_core::trace_info!("[Modpack] Resolving archive URL for {source_desc}");

    let archive_url = resolve_archive_url(source).await?;

    // Stable cache filename keyed by SHA1 of the resolved URL.
    let url_sha1 = calculate_sha1_bytes(archive_url.as_bytes());
    let cache_dir = AppState::cache_dir().join("modpacks");
    tokio::fs::create_dir_all(&cache_dir).await?;
    let archive_path = cache_dir.join(format!("{url_sha1}.archive"));
    let marker_path = cache_dir.join(format!("{url_sha1}.installed"));
    let work_dir = cache_dir.join(format!("work-{url_sha1}"));

    // Idempotence: marker file matches the URL hash means we're done.
    if marker_path.exists() {
        if let Ok(saved) = tokio::fs::read_to_string(&marker_path).await {
            if saved.trim() == url_sha1 {
                lighty_core::trace_info!(
                    "[Modpack] Already installed (marker matches), skipping"
                );
                return Ok(Vec::new());
            }
        }
    }

    if !archive_path.exists() {
        lighty_core::trace_info!("[Modpack] Downloading archive to {}", archive_path.display());
        download_file_untracked(&archive_url, &archive_path)
            .await
            .map_err(|e| InstallerError::DownloadFailed(format!("modpack archive: {e}")))?;
    }

    let archive_bytes = tokio::fs::metadata(&archive_path).await?.len();
    #[cfg(feature = "events")]
    if let Some(bus) = event_bus {
        bus.emit(Event::Modloader(ModloaderEvent::ModpackArchiveDownloaded {
            sha1: url_sha1.clone(),
            bytes: archive_bytes,
        }));
    }

    if work_dir.exists() {
        tokio::fs::remove_dir_all(&work_dir).await?;
    }
    tokio::fs::create_dir_all(&work_dir).await?;
    extract_zip(&archive_path, &work_dir).await?;

    let mods = if work_dir.join("modrinth.index.json").exists() {
        dispatch_mrpack(&work_dir, version).await?
    } else if work_dir.join("manifest.json").exists() {
        dispatch_cf_modpack(&work_dir, version).await?
    } else {
        return Err(InstallerError::Query(QueryError::UnsupportedFormat {
            what: "modpack archive".to_string(),
            expected: "modrinth.index.json or manifest.json".to_string(),
            found: "no recognised manifest".to_string(),
        }));
    };

    tokio::fs::write(&marker_path, &url_sha1).await?;

    if work_dir.exists() {
        let _ = tokio::fs::remove_dir_all(&work_dir).await;
    }

    #[cfg(feature = "events")]
    if let Some(bus) = event_bus {
        bus.emit(Event::Modloader(ModloaderEvent::ModpackInstalled {
            name: source_desc.clone(),
            mods_count: mods.len(),
        }));
    }

    lighty_core::trace_info!(
        "[Modpack] Install complete: {} mods queued for download ({} bytes archive)",
        mods.len(),
        archive_bytes
    );

    Ok(mods)
}

async fn resolve_archive_url(source: &ModpackSource) -> InstallerResult<String> {
    match source {
        #[cfg(feature = "modrinth")]
        ModpackSource::ModrinthUrl(_) | ModpackSource::ModrinthPinned { .. } => {
            Ok(lighty_modsloader::modrinth::modpack::resolve_mrpack_url(source).await?)
        }
        #[cfg(feature = "curseforge")]
        ModpackSource::CurseForgePinned { .. } => {
            Ok(lighty_modsloader::curseforge::modpack::resolve_cf_modpack_url(source).await?)
        }
        #[allow(unreachable_patterns)]
        _ => Err(InstallerError::Query(QueryError::Conversion {
            message: "Modpack source does not match any enabled provider feature".into(),
        })),
    }
}

fn describe_source(source: &ModpackSource) -> String {
    match source {
        #[cfg(feature = "modrinth")]
        ModpackSource::ModrinthUrl(url) => format!("ModrinthUrl({url})"),
        #[cfg(feature = "modrinth")]
        ModpackSource::ModrinthPinned { project, version } => match version {
            Some(v) => format!("ModrinthPinned({project}@{v})"),
            None => format!("ModrinthPinned({project}@latest)"),
        },
        #[cfg(feature = "curseforge")]
        ModpackSource::CurseForgePinned { project_id, file_id } => {
            format!("CurseForgePinned({project_id}/{file_id})")
        }
    }
}

/// Synchronous ZIP extraction wrapped in `spawn_blocking`.
async fn extract_zip(archive_path: &Path, dest: &Path) -> InstallerResult<()> {
    let archive_path = archive_path.to_path_buf();
    let dest = dest.to_path_buf();
    let archive_label = archive_path.display().to_string();
    let archive_label_for_open = archive_label.clone();
    let archive_label_for_extract = archive_label.clone();
    tokio::task::spawn_blocking(move || -> InstallerResult<()> {
        let file = std::fs::File::open(&archive_path)?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| {
            InstallerError::Query(QueryError::ArchiveCorrupted {
                context: format!("modpack archive {archive_label_for_open}"),
                reason: e.to_string(),
            })
        })?;
        archive.extract(&dest).map_err(|e| {
            InstallerError::Query(QueryError::ArchiveCorrupted {
                context: format!("modpack archive {archive_label_for_extract}"),
                reason: e.to_string(),
            })
        })?;
        Ok(())
    })
    .await
    .map_err(|e| {
        InstallerError::Query(QueryError::ArchiveCorrupted {
            context: format!("modpack archive {archive_label}"),
            reason: format!("extract task panicked: {e}"),
        })
    })??;
    Ok(())
}

async fn dispatch_mrpack<V: VersionInfo>(
    work_dir: &Path,
    version: &V,
) -> InstallerResult<Vec<Mods>> {
    #[cfg(feature = "modrinth")]
    {
        super::modrinth::process_mrpack(work_dir, version).await
    }
    #[cfg(not(feature = "modrinth"))]
    {
        let _ = (work_dir, version);
        Err(InstallerError::Query(QueryError::Conversion {
            message: "Found a Modrinth .mrpack but the `modrinth` feature is disabled".into(),
        }))
    }
}

async fn dispatch_cf_modpack<V: VersionInfo>(
    work_dir: &Path,
    version: &V,
) -> InstallerResult<Vec<Mods>> {
    #[cfg(feature = "curseforge")]
    {
        super::curseforge::process_cf_modpack(work_dir, version).await
    }
    #[cfg(not(feature = "curseforge"))]
    {
        let _ = (work_dir, version);
        Err(InstallerError::Query(QueryError::Conversion {
            message: "Found a CurseForge modpack but the `curseforge` feature is disabled".into(),
        }))
    }
}
