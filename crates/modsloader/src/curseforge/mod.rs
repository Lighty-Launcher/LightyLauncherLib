// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! CurseForge integration: Core-API client (mods) and modpack `.zip` parser.

pub mod api;
pub mod client;
pub mod client_metadata;
pub mod modpack;
pub mod modpack_metadata;

pub use api::set_api_key;
pub use client::{fetch, fetch_pinned_file, CurseForgeKey, CURSEFORGE_CACHE};
