// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! CurseForge modpack `.zip` parser + archive URL resolver.

use std::path::Path;

use lighty_core::QueryError;

use super::client::fetch_pinned_file;
use super::modpack_metadata::CfModpackManifest;
use crate::modpack::ModpackSource;

/// Resolves a CurseForge modpack source into the `.zip` download URL.
/// Requires [`crate::curseforge::set_api_key`] to have been called.
pub async fn resolve_cf_modpack_url(source: &ModpackSource) -> Result<String, QueryError> {
    match source {
        ModpackSource::CurseForgePinned { project_id, file_id } => {
            let file = fetch_pinned_file(*project_id, *file_id).await?;
            file.download_url
                .ok_or_else(|| QueryError::ModDistributionForbidden {
                    id: format!("{project_id}/{file_id}"),
                })
        }
        _ => Err(QueryError::Conversion {
            message: "resolve_cf_modpack_url called with a non-CurseForge source".into(),
        }),
    }
}

/// Reads + deserializes `manifest.json` from an extracted archive.
pub fn parse_manifest(work_dir: &Path) -> Result<CfModpackManifest, QueryError> {
    let manifest_path = work_dir.join("manifest.json");
    let file = std::fs::File::open(&manifest_path)?;
    let manifest: CfModpackManifest = serde_json::from_reader(file)?;
    if manifest.manifest_type != "minecraftModpack" {
        return Err(QueryError::UnsupportedFormat {
            what: "CurseForge manifestType".to_string(),
            expected: "minecraftModpack".to_string(),
            found: manifest.manifest_type.clone(),
        });
    }
    if manifest.manifest_version != 1 {
        return Err(QueryError::UnsupportedFormat {
            what: "CurseForge manifestVersion".to_string(),
            expected: "1".to_string(),
            found: manifest.manifest_version.to_string(),
        });
    }
    Ok(manifest)
}
