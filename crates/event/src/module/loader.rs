// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Loader events (Vanilla, Fabric, Forge, etc.).

use serde::{Deserialize, Serialize};

/// Loader events (Vanilla, Fabric, Forge, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum LoaderEvent {
    FetchingData {
        loader: String,
        minecraft_version: String,
        loader_version: String,
    },
    DataFetched {
        loader: String,
        minecraft_version: String,
        loader_version: String,
    },
    ManifestNotFound {
        loader: String,
        minecraft_version: String,
        loader_version: String,
        error: String,
    },
    ManifestCached {
        loader: String,
    },
    MergingLoaderData {
        base_loader: String,
        overlay_loader: String,
    },
    DataMerged {
        base_loader: String,
        overlay_loader: String,
    },
}
