// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Modrinth integration: Labrinth-API client (mods) and `.mrpack` parser.

pub mod api;
pub mod client;
pub mod client_metadata;
pub mod modpack;
pub mod modpack_metadata;

pub use client::{fetch, ModrinthKey, MODRINTH_CACHE};
