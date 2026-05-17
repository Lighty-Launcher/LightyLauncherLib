use crate::types::version_metadata::Version;
use crate::types::VersionInfo;
use lighty_core::QueryError;
use async_trait::async_trait;
use std::hash::Hash;

pub type Result<T> = std::result::Result<T, QueryError>;

/// Generic loader manifest query interface.
///
/// Implementors describe a loader's manifest source and how to extract
/// each sub-query (libraries, main class, etc.) from the raw payload.
/// `ManifestRepository<F>` handles caching and concurrency on top.
#[async_trait]
pub trait Query: Send + Sync {
    type Query: Eq + Hash + Clone + Send + Sync + 'static;

    type Data: Clone + Send + Sync + 'static;

    type Raw: Send + Sync + 'static;


    /// Loader group name (`"vanilla"`, `"forge"`, `"custom"`, ...).
    fn name() -> &'static str;

    /// Fetches the raw manifest from its remote source.
    async fn fetch_full_data<V: VersionInfo>(version: &V) -> Result<Self::Raw>;

    /// Extracts a typed sub-query from the raw manifest.
    async fn extract<V: VersionInfo>(version: &V, query: &Self::Query, raw: &Self::Raw) -> Result<Self::Data>;

    /// Builds the full [`Version`] (all sub-queries merged) from the raw manifest.
    async fn version_builder<V: VersionInfo>(version: &V, full_data: &Self::Raw) -> Result<Version>;
}


/// Cache key combining instance name and sub-query discriminator.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QueryKey<Q> {
    pub version: String,
    pub query: Q,
}

