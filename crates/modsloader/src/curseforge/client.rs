// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! CurseForge Core-API client. Requires
//! [`set_api_key`](super::api::set_api_key) before any [`fetch`]. Results
//! are cached process-wide in [`CURSEFORGE_CACHE`].

use std::sync::Arc;
use std::time::Duration;

use once_cell::sync::Lazy;

use lighty_core::hosts::HTTP_CLIENT as CLIENT;

use lighty_loaders::types::version_metadata::Mods;
use lighty_loaders::types::Loader;
use lighty_core::QueryError;
use lighty_loaders::utils::cache::Cache;

use super::api::{read_api_key, url_encode, BASE_URL, PROVIDER};
use super::client_metadata::{
    CurseForgeDependency, CurseForgeEnvelope, CurseForgeFile, CurseForgeMod, CLASS_MOD,
    CLASS_RESOURCE_PACK, CLASS_SHADER_PACK, DEP_REQUIRED, HASH_ALGO_SHA1, MOD_LOADER_FABRIC,
    MOD_LOADER_FORGE, MOD_LOADER_NEOFORGE, MOD_LOADER_QUILT,
};
use crate::request::ModRequest;

/// Cache key — granularity = anything that changes the API response.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CurseForgeKey {
    pub mod_id: u32,
    pub pin: Option<u32>,
    pub mc: String,
    pub loader: Loader,
}

/// Process-wide cache of resolved CurseForge fetches.
pub static CURSEFORGE_CACHE: Lazy<Cache<CurseForgeKey, Arc<(Mods, Vec<ModRequest>)>>> =
    Lazy::new(Cache::with_smart_cleanup);

/// Class-id cache keyed by `mod_id`. CurseForge's `classId` never changes
/// for a given project, so a single round-trip is amortized.
static CLASS_ID_CACHE: Lazy<Cache<u32, Arc<u32>>> = Lazy::new(Cache::with_smart_cleanup);

/// Resolves a `ModRequest::CurseForge { … }` into a pivot [`Mods`]
/// entry plus the list of required dependencies (relation type 3) to
/// enqueue.
pub async fn fetch(
    request: &ModRequest,
    minecraft_version: &str,
    loader: &Loader,
    ttl: Duration,
) -> Result<(Mods, Vec<ModRequest>), QueryError> {
    let (mod_id, pinned_file) = match request {
        ModRequest::CurseForge { mod_id, file_id } => (*mod_id, *file_id),
        _ => unreachable!("curseforge::fetch called with a non-CurseForge request"),
    };

    let key = CurseForgeKey {
        mod_id,
        pin: pinned_file,
        mc: minecraft_version.to_string(),
        loader: loader.clone(),
    };
    let loader_for_fetch = loader.clone();

    let cached = CURSEFORGE_CACHE
        .get_or_try_insert_with(key, ttl, || async move {
            let api_key = read_api_key()?;
            let subdir = install_subdir(class_id(mod_id, &api_key, ttl).await?)?;
            let file = if let Some(file_id) = pinned_file {
                fetch_pinned_file_with_key(mod_id, file_id, &api_key).await?
            } else {
                let mod_loader_code = mod_loader_code(&loader_for_fetch)?;
                fetch_latest_compatible(mod_id, minecraft_version, mod_loader_code, &api_key)
                    .await?
            };
            let pivot = into_pivot(&file, subdir)?;
            let dependencies = collect_required_dependencies(&file);
            Ok::<_, QueryError>(Arc::new((pivot, dependencies)))
        })
        .await?;

    Ok((*cached).clone())
}

/// Maps a [`Loader`] to its CurseForge numeric `modLoaderType` code.
fn mod_loader_code(loader: &Loader) -> Result<u8, QueryError> {
    match loader {
        Loader::Fabric => Ok(MOD_LOADER_FABRIC),
        Loader::Forge => Ok(MOD_LOADER_FORGE),
        Loader::NeoForge => Ok(MOD_LOADER_NEOFORGE),
        Loader::Quilt => Ok(MOD_LOADER_QUILT),
        Loader::Vanilla | Loader::Optifine | Loader::LightyUpdater => {
            Err(QueryError::UnsupportedLoader(format!(
                "CurseForge doesn't host mods for {:?} instances",
                loader
            )))
        }
    }
}

async fn fetch_pinned_file_with_key(
    mod_id: u32,
    file_id: u32,
    api_key: &str,
) -> Result<CurseForgeFile, QueryError> {
    let url = format!("{}/mods/{}/files/{}", BASE_URL, mod_id, file_id);
    let response = CLIENT.get(&url).header("x-api-key", api_key).send().await?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(QueryError::ModNotFound {
            provider: PROVIDER,
            id: format!("{}/{}", mod_id, file_id),
        });
    }
    let envelope = response
        .error_for_status()?
        .json::<CurseForgeEnvelope<CurseForgeFile>>()
        .await?;
    Ok(envelope.data)
}

/// Public wrapper around the pinned-file endpoint that reads the API key
/// from the process-wide [`set_api_key`](super::api::set_api_key) store.
pub async fn fetch_pinned_file(
    mod_id: u32,
    file_id: u32,
) -> Result<CurseForgeFile, QueryError> {
    let api_key = read_api_key()?;
    fetch_pinned_file_with_key(mod_id, file_id, &api_key).await
}

async fn fetch_latest_compatible(
    mod_id: u32,
    minecraft_version: &str,
    mod_loader_code: u8,
    api_key: &str,
) -> Result<CurseForgeFile, QueryError> {
    let url = format!(
        "{}/mods/{}/files?gameVersion={}&modLoaderType={}",
        BASE_URL,
        mod_id,
        url_encode(minecraft_version),
        mod_loader_code
    );

    let response = CLIENT.get(&url).header("x-api-key", api_key).send().await?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(QueryError::ModNotFound {
            provider: PROVIDER,
            id: mod_id.to_string(),
        });
    }
    let envelope = response
        .error_for_status()?
        .json::<CurseForgeEnvelope<Vec<CurseForgeFile>>>()
        .await?;
    envelope
        .data
        .into_iter()
        .next()
        .ok_or_else(|| QueryError::ModIncompatible {
            provider: PROVIDER,
            id: mod_id.to_string(),
            mc: minecraft_version.to_string(),
            loader: mod_loader_code.to_string(),
        })
}

/// Converts a CurseForge file entry into the launcher's pivot [`Mods`].
/// `subdir` is the runtime-relative folder picked from the project's
/// `classId` (`"mods"`, `"resourcepacks"`, `"shaderpacks"`).
fn into_pivot(file: &CurseForgeFile, subdir: &str) -> Result<Mods, QueryError> {
    let url = file
        .download_url
        .clone()
        .ok_or_else(|| QueryError::ModDistributionForbidden {
            id: file.mod_id.to_string(),
        })?;
    let sha1 = file
        .hashes
        .iter()
        .find(|h| h.algo == HASH_ALGO_SHA1)
        .map(|h| h.value.clone());
    Ok(Mods {
        name: format!("{}-{}", file.mod_id, file.id),
        url: Some(url),
        path: Some(format!("{}/{}", subdir, file.file_name)),
        sha1,
        size: (file.file_length > 0).then_some(file.file_length),
    })
}

/// Maps a CurseForge `classId` to the runtime sub-folder used by the
/// installer. Unknown classes surface a clear `UnsupportedFormat` error.
fn install_subdir(class_id: u32) -> Result<&'static str, QueryError> {
    match class_id {
        CLASS_MOD => Ok("mods"),
        CLASS_RESOURCE_PACK => Ok("resourcepacks"),
        CLASS_SHADER_PACK => Ok("shaderpacks"),
        other => Err(QueryError::UnsupportedFormat {
            what: "CurseForge classId".to_string(),
            expected: format!(
                "{} (mod) | {} (resourcepack) | {} (shader)",
                CLASS_MOD, CLASS_RESOURCE_PACK, CLASS_SHADER_PACK
            ),
            found: other.to_string(),
        }),
    }
}

/// Fetches `/mods/{mod_id}` and returns its `classId`. Memoized in
/// [`CLASS_ID_CACHE`] for `ttl` to amortize the extra round-trip across
/// file fetches and modpack file resolutions.
pub(crate) async fn class_id(
    mod_id: u32,
    api_key: &str,
    ttl: Duration,
) -> Result<u32, QueryError> {
    let api_key = api_key.to_string();
    let cached = CLASS_ID_CACHE
        .get_or_try_insert_with(mod_id, ttl, || async move {
            let url = format!("{}/mods/{}", BASE_URL, mod_id);
            let response = CLIENT.get(&url).header("x-api-key", &api_key).send().await?;
            if response.status() == reqwest::StatusCode::NOT_FOUND {
                return Err(QueryError::ModNotFound {
                    provider: PROVIDER,
                    id: mod_id.to_string(),
                });
            }
            let envelope = response
                .error_for_status()?
                .json::<CurseForgeEnvelope<CurseForgeMod>>()
                .await?;
            Ok::<_, QueryError>(Arc::new(envelope.data.class_id))
        })
        .await?;
    Ok(*cached)
}

/// Convenience helper for callers outside this module (e.g. the modpack
/// pipeline) that need the install subdir but not the full fetch flow.
/// Returns `"mods"` / `"resourcepacks"` / `"shaderpacks"`, or
/// `UnsupportedFormat` if the project's classId is unknown.
pub async fn install_subdir_for(mod_id: u32, ttl: Duration) -> Result<&'static str, QueryError> {
    let api_key = read_api_key()?;
    install_subdir(class_id(mod_id, &api_key, ttl).await?)
}

/// Picks every `required` dependency (relation type 3), drops the rest.
fn collect_required_dependencies(file: &CurseForgeFile) -> Vec<ModRequest> {
    file.dependencies
        .iter()
        .filter(|dep| dep.relation_type == DEP_REQUIRED)
        .map(modrequest_from)
        .collect()
}

fn modrequest_from(dep: &CurseForgeDependency) -> ModRequest {
    ModRequest::CurseForge {
        mod_id: dep.mod_id,
        file_id: None,
    }
}
