// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Per-instance version description.
//!
//! - [`version_info`] — the `VersionInfo` trait used by every builder.
//! - [`version_metadata`] — pivot metadata produced by loader queries
//!   and consumed by the install + launch pipelines.

pub mod version_info;
pub mod version_metadata;
