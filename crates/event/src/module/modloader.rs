// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Mod-source events: dependency resolution, modpack pipeline,
//! per-bucket install summaries (resourcepacks, shaderpacks, datapacks).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum ModloaderEvent {
    ResolveStarted {
        request_count: usize,
    },
    ResolveFetching {
        source: String,
        identifier: String,
    },
    ResolveDependency {
        parent: String,
        dependency: String,
    },
    ResolveCompleted {
        total_mods: usize,
    },
    ModpackResolveStart {
        source: String,
    },
    ModpackArchiveDownloaded {
        sha1: String,
        bytes: u64,
    },
    ModpackOverridesExtracted {
        count: usize,
    },
    ModpackInstalled {
        name: String,
        mods_count: usize,
    },
    ResourcePacksInstalled {
        count: usize,
        bytes: u64,
    },
    ShaderPacksInstalled {
        count: usize,
        bytes: u64,
    },
    DatapacksInstalled {
        count: usize,
        bytes: u64,
    },
}
