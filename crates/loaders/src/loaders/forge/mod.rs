//! Forge loader module — covers both modern (≥ 1.13) and legacy
//! (1.4 → 1.12.2) Forge installers behind the single
//! [`crate::types::Loader::Forge`] variant.

pub mod forge;
pub mod forge_legacy;
pub mod forge_legacy_metadata;
