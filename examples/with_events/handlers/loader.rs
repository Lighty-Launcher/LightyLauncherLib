//! Loader-metadata fetch + merge events (Vanilla, Fabric, Forge, …).

use lighty_launcher::prelude::*;

pub fn log(event: LoaderEvent) {
    match event {
        LoaderEvent::FetchingData { loader, minecraft_version, loader_version } => {
            trace_info!(
                "[Loader] Fetching {} data for Minecraft {} (loader version: {})",
                loader, minecraft_version, loader_version
            );
        }
        LoaderEvent::DataFetched { loader, .. } => {
            trace_info!("[Loader] {} data fetched successfully", loader);
        }
        LoaderEvent::ManifestCached { loader } => {
            trace_info!("[Loader] Using cached {} manifest", loader);
        }
        LoaderEvent::MergingLoaderData { base_loader, overlay_loader } => {
            trace_info!("[Loader] Merging {} with {}", overlay_loader, base_loader);
        }
        LoaderEvent::DataMerged { base_loader, overlay_loader } => {
            trace_info!("[Loader] {} and {} merged successfully", overlay_loader, base_loader);
        }
        _ => {}
    }
}
