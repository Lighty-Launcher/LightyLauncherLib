use crate::types::version_metadata::{Version, VersionMetaData};
use crate::types::{Loader, LoaderExtensions, VersionInfo};
use lighty_core::QueryError;

pub type Result<T> = std::result::Result<T, QueryError>;

/// Fetches the base loader's metadata for already-resolved coordinates.
///
/// The Lighty overrides are layered on top by the caller, which is why this
/// returns the base [`Version`] untouched.
pub async fn merge_metadata<V: VersionInfo<LoaderType = Loader>>(version: &V) -> Result<Version> {
    let metadata = version.get_metadata().await?;

    match &*metadata {
        VersionMetaData::Version(merged) => Ok(merged.clone()),
        _ => Err(QueryError::InvalidMetadata),
    }
}
