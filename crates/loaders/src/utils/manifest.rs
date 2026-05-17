use lighty_core::QueryError;
use crate::utils::query::{Query, QueryKey};
use crate::utils::cache::Cache;
use crate::types::VersionInfo;
use std::sync::Arc;

pub type Result<T> = std::result::Result<T, QueryError>;

/// Caches `Query` results so each remote manifest is fetched at most
/// once per TTL window.
///
/// `raw_version_cache` holds the raw manifest returned by
/// [`Query::fetch_full_data`] keyed by instance name; `query_cache`
/// holds extracted sub-query results keyed by `(instance, sub-query)`.
pub struct ManifestRepository<F: Query> {
    query_cache: Arc<Cache<QueryKey<F::Query>, Arc<F::Data>>>,
    raw_version_cache: Arc<Cache<String, Arc<<F as Query>::Raw>>>,
    _marker: std::marker::PhantomData<F>,
}

impl<F: Query> ManifestRepository<F> {
    /// Creates an empty repository with smart-cleanup-enabled caches.
    pub fn new() -> Self {
        Self {
            query_cache: Arc::new(Cache::with_smart_cleanup()),
            raw_version_cache: Arc::new(Cache::with_smart_cleanup()),
            _marker: std::marker::PhantomData,
        }
    }

    pub async fn get<V: VersionInfo>(
        &self,
        version: &V,
        query: F::Query,
    ) -> Result<Arc<F::Data>> {
        let key = QueryKey {
            version: version.name().to_string(),
            query: query.clone(),
        };

        let ttl = version.ttl();

        let manifest_data: Arc<F::Data> = self
            .query_cache
            .get_or_try_insert_with(key.clone(), ttl, || {
                let version = version.clone();
                let query = query.clone();
                let repo = self.clone();

                async move {
                    let full_data = repo.get_cached_version_data(&version).await?;
                    let data = F::extract(&version, &query, &full_data).await?;
                    Ok::<Arc<F::Data>, QueryError>(Arc::new(data))
                }
            })
            .await?;

        Ok(manifest_data)
    }

    pub async fn get_raw<V: VersionInfo>(&self, version: &V) -> Result<Arc<<F as Query>::Raw>> {
        self.get_cached_version_data(version).await
    }
    
    async fn get_cached_version_data<V: VersionInfo>(&self, version: &V) -> Result<Arc<<F as Query>::Raw>> {
        let ttl = version.ttl();
        let key = version.name().to_string();

        let data = self
            .raw_version_cache
            .get_or_try_insert_with(key.clone(), ttl, || {
                let version = version.clone();
                async move {
                    F::fetch_full_data(&version).await.map(Arc::new)
                }
            })
            .await?;

        Ok(data)
    }

    pub async fn clear_cache(&self) {
        self.query_cache.clear().await;
        self.raw_version_cache.clear().await;
    }

    /// Invalidates every cached entry (raw manifest + sub-queries) tied
    /// to a single instance. Used by `InstanceCache::clear_cache` to
    /// purge one builder without touching others.
    pub async fn invalidate(&self, version_name: &str) {
        let needle = version_name.to_string();
        let needle_for_query = needle.clone();
        self.raw_version_cache.retain(move |k| k != &needle).await;
        self.query_cache
            .retain(move |k| k.version != needle_for_query)
            .await;
    }

    pub async fn cache_len(&self) -> (usize, usize) {
        let queries = self.query_cache.len().await;
        let raw = self.raw_version_cache.len().await;
        (queries, raw)
    }

    pub async fn is_empty(&self) -> bool {
        self.query_cache.is_empty().await && self.raw_version_cache.is_empty().await
    }
}

impl<F: Query> Clone for ManifestRepository<F> {
    fn clone(&self) -> Self {
        Self {
            query_cache: Arc::clone(&self.query_cache),
            raw_version_cache: Arc::clone(&self.raw_version_cache),
            _marker: std::marker::PhantomData,
        }
    }
}