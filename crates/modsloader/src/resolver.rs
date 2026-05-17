// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Source-agnostic level-order BFS mod resolver. Each BFS depth is fetched
//! concurrently via `try_join_all`; `required` dependencies feed the next level.

use std::collections::HashSet;
use std::mem;
use std::time::Duration;

use futures::future::try_join_all;
use lighty_core::QueryError;
use lighty_loaders::types::version_metadata::Mods;
use lighty_loaders::types::Loader;

#[cfg(feature = "events")]
use lighty_event::{Event, EventBus, ModloaderEvent};

use super::request::{ModKey, ModRequest};

/// Resolves every user request + its transitive `required` dependencies
/// into a flat list of pivot [`Mods`] entries, deduplicated by
/// `(source, project-id)`.
///
/// `ttl` is the TTL applied to each cached entry — read from
/// `VersionInfo::ttl()` on the calling instance, so a single override on
/// the builder propagates to the per-provider caches uniformly.
///
/// With the `events` feature on, the four `ModloaderEvent::Resolve*`
/// variants surface on the supplied `EventBus` (or `None` to silence).
pub async fn resolve(
    requests: &[ModRequest],
    mc: &str,
    loader: &Loader,
    ttl: Duration,
    #[cfg(feature = "events")] event_bus: Option<&EventBus>,
) -> Result<Vec<Mods>, QueryError> {
    #[cfg(feature = "events")]
    if let Some(bus) = event_bus {
        bus.emit(Event::Modloader(ModloaderEvent::ResolveStarted {
            request_count: requests.len(),
        }));
    }

    let mut visited: HashSet<ModKey> = HashSet::new();
    let mut out: Vec<Mods> = Vec::new();

    // Seed the first level with the user requests, filtering duplicates
    // upfront so identical entries don't fire twice.
    let mut current_level: Vec<ModRequest> = requests
        .iter()
        .cloned()
        .filter(|req| visited.insert(ModKey::from(req)))
        .collect();

    while !current_level.is_empty() {
        for req in &current_level {
            let key = ModKey::from(req);
            lighty_core::trace_debug!(
                provider = %key.source.as_str(),
                ident = %key.id,
                "Fetching mod"
            );
            #[cfg(feature = "events")]
            if let Some(bus) = event_bus {
                bus.emit(Event::Modloader(ModloaderEvent::ResolveFetching {
                    source: key.source.as_str().to_string(),
                    identifier: key.id.clone(),
                }));
            }
        }

        let results: Vec<(Mods, Vec<ModRequest>)> = try_join_all(
            current_level
                .iter()
                .map(|req| fetch_one(req, mc, loader, ttl)),
        )
        .await?;

        let _ = mem::take(&mut current_level);

        for (pivot, deps) in results {
            let parent_name = pivot.name.clone();
            out.push(pivot);

            for dep in deps {
                let dep_key = ModKey::from(&dep);
                if !visited.insert(dep_key.clone()) {
                    continue;
                }
                lighty_core::trace_debug!(
                    parent = %parent_name,
                    dep = %dep_key.id,
                    "Enqueueing required mod dep"
                );
                #[cfg(feature = "events")]
                if let Some(bus) = event_bus {
                    bus.emit(Event::Modloader(ModloaderEvent::ResolveDependency {
                        parent: parent_name.clone(),
                        dependency: dep_key.id.clone(),
                    }));
                }
                current_level.push(dep);
            }
        }
    }

    #[cfg(feature = "events")]
    if let Some(bus) = event_bus {
        bus.emit(Event::Modloader(ModloaderEvent::ResolveCompleted {
            total_mods: out.len(),
        }));
    }

    Ok(out)
}

/// Single-request dispatch — picks the right client. A request for a
/// disabled source is rejected at runtime with a clear error.
async fn fetch_one(
    req: &ModRequest,
    mc: &str,
    loader: &Loader,
    ttl: Duration,
) -> Result<(Mods, Vec<ModRequest>), QueryError> {
    match req {
        #[cfg(feature = "modrinth")]
        ModRequest::Modrinth { .. } => super::modrinth::fetch(req, mc, loader, ttl).await,

        #[cfg(not(feature = "modrinth"))]
        ModRequest::Modrinth { id_or_slug, .. } => {
            let _ = ttl;
            Err(QueryError::UnsupportedLoader(format!(
                "Modrinth support is disabled (cargo feature 'modrinth' not enabled) — \
                 cannot fetch '{}'",
                id_or_slug
            )))
        }

        #[cfg(feature = "curseforge")]
        ModRequest::CurseForge { .. } => super::curseforge::fetch(req, mc, loader, ttl).await,

        #[cfg(not(feature = "curseforge"))]
        ModRequest::CurseForge { mod_id, .. } => {
            let _ = ttl;
            Err(QueryError::UnsupportedLoader(format!(
                "CurseForge support is disabled (cargo feature 'curseforge' not enabled) — \
                 cannot fetch mod #{}",
                mod_id
            )))
        }
    }
}
