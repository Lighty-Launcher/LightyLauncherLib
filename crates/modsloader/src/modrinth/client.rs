// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Modrinth Labrinth-API client. Results are cached process-wide in
//! [`MODRINTH_CACHE`]; invalidate via
//! [`InstanceCache::clear_cache`](crate::InstanceCache::clear_cache).

use std::sync::Arc;
use std::time::Duration;

use lighty_core::hosts::HTTP_CLIENT as CLIENT;

use lighty_loaders::types::version_metadata::Mods;
use lighty_loaders::types::Loader;
use lighty_core::QueryError;
use lighty_loaders::utils::cache::Cache;
use once_cell::sync::Lazy;

use super::api::{url_encode, BASE_URL, PROVIDER, USER_AGENT};
use super::client_metadata::{ModrinthDependency, ModrinthFile, ModrinthProject, ModrinthVersion};
use crate::request::ModRequest;

/// Cache key — granularity = anything that changes the API response.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModrinthKey {
    pub id_or_slug: String,
    pub pin: Option<String>,
    pub mc: String,
    pub loader: Loader,
}

/// Process-wide cache of resolved Modrinth fetches.
pub static MODRINTH_CACHE: Lazy<Cache<ModrinthKey, Arc<(Mods, Vec<ModRequest>)>>> =
    Lazy::new(Cache::with_smart_cleanup);

/// Project-type cache keyed by slug/id. The type never changes for a
/// given project, so a single round-trip is amortized across every
/// version fetched for that project.
static PROJECT_TYPE_CACHE: Lazy<Cache<String, Arc<String>>> =
    Lazy::new(Cache::with_smart_cleanup);

/// Resolves a single `ModRequest::Modrinth { … }` into a pivot
/// [`Mods`] entry + the list of `required` dependencies to enqueue.
pub async fn fetch(
    request: &ModRequest,
    minecraft_version: &str,
    loader: &Loader,
    ttl: Duration,
) -> Result<(Mods, Vec<ModRequest>), QueryError> {
    let (id_or_slug, pinned_version) = match request {
        ModRequest::Modrinth { id_or_slug, version } => (id_or_slug.clone(), version.clone()),
        _ => unreachable!("modrinth::fetch called with a non-Modrinth request"),
    };

    let key = ModrinthKey {
        id_or_slug: id_or_slug.clone(),
        pin: pinned_version.clone(),
        mc: minecraft_version.to_string(),
        loader: loader.clone(),
    };
    let loader_for_fetch = loader.clone();

    let cached = MODRINTH_CACHE
        .get_or_try_insert_with(key, ttl, || async move {
            let subdir = install_subdir(&project_type(&id_or_slug, ttl).await?)?;
            let version = if let Some(version_id) = pinned_version {
                fetch_pinned_version(&version_id).await?
            } else {
                let loader_tag = loader_tag(&loader_for_fetch)?;
                fetch_latest_compatible(&id_or_slug, minecraft_version, loader_tag).await?
            };
            let pivot = into_pivot(&id_or_slug, &version, subdir)?;
            let dependencies = collect_required_dependencies(&version);
            Ok::<_, QueryError>(Arc::new((pivot, dependencies)))
        })
        .await?;

    // The cache stores Arc<(Mods, Vec<ModRequest>)>; the resolver consumes
    // an owned tuple. Single Arc clone, then clone the inner data once.
    Ok((*cached).clone())
}

/// Maps a [`Loader`] to its Modrinth loader-tag string.
fn loader_tag(loader: &Loader) -> Result<&'static str, QueryError> {
    match loader {
        Loader::Fabric => Ok("fabric"),
        Loader::Forge => Ok("forge"),
        Loader::NeoForge => Ok("neoforge"),
        Loader::Quilt => Ok("quilt"),
        Loader::Vanilla | Loader::Optifine | Loader::LightyUpdater => {
            Err(QueryError::UnsupportedLoader(format!(
                "Modrinth doesn't host mods for {:?} instances",
                loader
            )))
        }
    }
}

async fn fetch_pinned_version(version_id: &str) -> Result<ModrinthVersion, QueryError> {
    let url = format!("{}/version/{}", BASE_URL, version_id);
    let response = CLIENT.get(&url).header("User-Agent", USER_AGENT).send().await?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(QueryError::ModNotFound {
            provider: PROVIDER,
            id: version_id.to_string(),
        });
    }
    Ok(response.error_for_status()?.json::<ModrinthVersion>().await?)
}

async fn fetch_latest_compatible(
    slug: &str,
    minecraft_version: &str,
    loader_tag: &str,
) -> Result<ModrinthVersion, QueryError> {
    // Query-string arrays use JSON encoding per Modrinth's docs.
    let loaders_param = url_encode(&format!("[\"{}\"]", loader_tag));
    let game_versions_param = url_encode(&format!("[\"{}\"]", minecraft_version));
    let url = format!(
        "{}/project/{}/version?loaders={}&game_versions={}",
        BASE_URL, slug, loaders_param, game_versions_param
    );

    let response = CLIENT.get(&url).header("User-Agent", USER_AGENT).send().await?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(QueryError::ModNotFound {
            provider: PROVIDER,
            id: slug.to_string(),
        });
    }
    let versions = response.error_for_status()?.json::<Vec<ModrinthVersion>>().await?;
    versions
        .into_iter()
        .next()
        .ok_or_else(|| QueryError::ModIncompatible {
            provider: PROVIDER,
            id: slug.to_string(),
            mc: minecraft_version.to_string(),
            loader: loader_tag.to_string(),
        })
}

/// Picks the binary file from a version and converts it into the pivot [`Mods`].
/// `subdir` is the runtime-relative folder (`"mods"`, `"resourcepacks"`, …).
fn into_pivot(slug: &str, version: &ModrinthVersion, subdir: &str) -> Result<Mods, QueryError> {
    let file = primary_file(&version.files).ok_or_else(|| QueryError::MissingField {
        field: format!("file in modrinth version {}", version.id),
    })?;
    Ok(Mods {
        name: format!("{}-{}", slug, version.version_number),
        url: Some(file.url.clone()),
        path: Some(format!("{}/{}", subdir, file.filename)),
        sha1: Some(file.hashes.sha1.clone()),
        size: Some(file.size),
    })
}

/// Maps a Modrinth `project_type` to the runtime sub-folder used by the
/// installer. Unknown types surface a clear `UnsupportedFormat` error.
fn install_subdir(project_type: &str) -> Result<&'static str, QueryError> {
    match project_type {
        "mod" => Ok("mods"),
        "resourcepack" => Ok("resourcepacks"),
        "shader" => Ok("shaderpacks"),
        "datapack" => {
            lighty_core::trace_warn!(
                "Modrinth datapack: routing to top-level datapacks/ — per-world install requires manual handling"
            );
            Ok("datapacks")
        }
        other => Err(QueryError::UnsupportedFormat {
            what: "Modrinth project_type".to_string(),
            expected: "mod | resourcepack | shader | datapack".to_string(),
            found: other.to_string(),
        }),
    }
}

/// Fetches `/project/{slug}` and returns its `project_type`. Results are
/// memoized in [`PROJECT_TYPE_CACHE`] for `ttl` to amortize the extra
/// round-trip across versions of the same project.
async fn project_type(id_or_slug: &str, ttl: Duration) -> Result<String, QueryError> {
    let key = id_or_slug.to_string();
    let slug = id_or_slug.to_string();
    let cached = PROJECT_TYPE_CACHE
        .get_or_try_insert_with(key, ttl, || async move {
            let url = format!("{}/project/{}", BASE_URL, slug);
            let response = CLIENT.get(&url).header("User-Agent", USER_AGENT).send().await?;
            if response.status() == reqwest::StatusCode::NOT_FOUND {
                return Err(QueryError::ModNotFound {
                    provider: PROVIDER,
                    id: slug,
                });
            }
            let project: ModrinthProject = response.error_for_status()?.json().await?;
            Ok::<_, QueryError>(Arc::new(project.project_type))
        })
        .await?;
    Ok((*cached).clone())
}

/// Selects the build artifact among a version's `files` array.
fn primary_file(files: &[ModrinthFile]) -> Option<&ModrinthFile> {
    files.iter().find(|f| f.primary).or_else(|| files.first())
}

/// Picks every `required` dependency of `version`, dropping the rest.
fn collect_required_dependencies(version: &ModrinthVersion) -> Vec<ModRequest> {
    version
        .dependencies
        .iter()
        .filter(|dep| dep.dependency_type == "required")
        .filter_map(|dep| modrequest_from(dep, &version.id))
        .collect()
}

fn modrequest_from(dep: &ModrinthDependency, parent_version: &str) -> Option<ModRequest> {
    let project_id = dep.project_id.clone().or_else(|| {
        lighty_core::trace_debug!(
            parent_version = %parent_version,
            "Skipping Modrinth dep with no project_id"
        );
        None
    })?;
    Some(ModRequest::Modrinth {
        id_or_slug: project_id,
        version: dep.version_id.clone(),
    })
}
