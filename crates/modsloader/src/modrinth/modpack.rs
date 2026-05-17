// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Modrinth `.mrpack` parser + archive URL resolver.

use std::path::Path;

use lighty_core::QueryError;
use lighty_core::hosts::HTTP_CLIENT as CLIENT;

use super::api::{BASE_URL, PROVIDER, USER_AGENT};
use super::client_metadata::ModrinthVersion;
use super::modpack_metadata::MrpackManifest;
use crate::modpack::ModpackSource;

/// Resolves a Modrinth modpack source into the `.mrpack` download URL.
/// Errors if `source` is not a Modrinth variant.
pub async fn resolve_mrpack_url(source: &ModpackSource) -> Result<String, QueryError> {
    match source {
        ModpackSource::ModrinthUrl(url) => Ok(url.clone()),

        ModpackSource::ModrinthPinned {
            project,
            version: Some(version_id),
        } => {
            let version = fetch_pinned_version(version_id).await?;
            primary_mrpack_url(&version, project)
        }

        ModpackSource::ModrinthPinned {
            project,
            version: None,
        } => {
            // /project/{slug}/version returns date-desc — take the first.
            let url = format!("{BASE_URL}/project/{project}/version");
            let response = CLIENT.get(&url).header("User-Agent", USER_AGENT).send().await?;
            if response.status() == reqwest::StatusCode::NOT_FOUND {
                return Err(QueryError::ModNotFound {
                    provider: PROVIDER,
                    id: project.clone(),
                });
            }
            let versions = response
                .error_for_status()?
                .json::<Vec<ModrinthVersion>>()
                .await?;
            let version = versions
                .into_iter()
                .next()
                .ok_or_else(|| QueryError::ModNotFound {
                    provider: PROVIDER,
                    id: project.clone(),
                })?;
            primary_mrpack_url(&version, project)
        }

        #[cfg(feature = "curseforge")]
        ModpackSource::CurseForgePinned { .. } => Err(QueryError::Conversion {
            message: "resolve_mrpack_url called with a CurseForge source".into(),
        }),
    }
}

async fn fetch_pinned_version(version_id: &str) -> Result<ModrinthVersion, QueryError> {
    let url = format!("{BASE_URL}/version/{version_id}");
    let response = CLIENT.get(&url).header("User-Agent", USER_AGENT).send().await?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(QueryError::ModNotFound {
            provider: PROVIDER,
            id: version_id.to_string(),
        });
    }
    Ok(response.error_for_status()?.json::<ModrinthVersion>().await?)
}

fn primary_mrpack_url(version: &ModrinthVersion, project: &str) -> Result<String, QueryError> {
    // Filter on `.mrpack` extension — a version can ship client + server packs.
    let file = version
        .files
        .iter()
        .find(|f| f.filename.ends_with(".mrpack"))
        .or_else(|| version.files.iter().find(|f| f.primary))
        .or_else(|| version.files.first())
        .ok_or_else(|| QueryError::MissingField {
            field: format!(".mrpack file in Modrinth version {} ({project})", version.id),
        })?;
    Ok(file.url.clone())
}

/// Reads + deserializes `modrinth.index.json` from an extracted archive.
pub fn parse_manifest(work_dir: &Path) -> Result<MrpackManifest, QueryError> {
    let index_path = work_dir.join("modrinth.index.json");
    let file = std::fs::File::open(&index_path)?;
    let manifest: MrpackManifest = serde_json::from_reader(file)?;
    if manifest.format_version != 1 {
        return Err(QueryError::UnsupportedFormat {
            what: ".mrpack formatVersion".to_string(),
            expected: "1".to_string(),
            found: manifest.format_version.to_string(),
        });
    }
    Ok(manifest)
}
