use once_cell::sync::Lazy;
use async_trait::async_trait;
use futures::future::join_all;
use std::collections::HashMap;

use lighty_core::hosts::HTTP_CLIENT as CLIENT;

use super::quilt_metadata::QuiltMetaData;
use crate::types::VersionInfo;

use crate::loaders::vanilla::vanilla::VanillaQuery;
use lighty_core::QueryError;
use crate::utils::{query::Query, manifest::ManifestRepository};
use crate::utils::maven::{fetch_file_size, fetch_maven_sha1};
use crate::types::version_metadata::
{Library, VersionMetaData, Arguments, MainClass, Version};


/// QuiltMC metadata server (returns the `profile/json` manifest).
const QUILT_META: &str = "https://meta.quiltmc.org/v3/versions/loader";

/// Shared cached repository for Quilt manifests.
pub static QUILT: Lazy<ManifestRepository<QuiltQuery>> = Lazy::new(|| ManifestRepository::new());

pub type Result<T> = std::result::Result<T, QueryError>;

/// Sub-queries supported by the Quilt loader.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum QuiltQuery {
    MainClass,
    Libraries,
    Arguments,
    QuiltBuilder,
}

#[async_trait]
impl Query for QuiltQuery {
    type Query = QuiltQuery;
    type Data = VersionMetaData;
    type Raw = QuiltMetaData;

    fn name() -> &'static str {
        "quilt"
    }

    async fn fetch_full_data<V: VersionInfo>(version: &V) -> Result<QuiltMetaData> {
        let manifest_url = format!(
            "{}/{}/{}/profile/json",
            QUILT_META,
            version.minecraft_version(),
            version.loader_version()
        );
        lighty_core::trace_debug!(url = %manifest_url, loader = "quilt", "Fetching manifest");
        let manifest: QuiltMetaData = CLIENT.get(manifest_url).send().await?.json().await?;
        Ok(manifest)
    }

    async fn extract<V: VersionInfo>(version: &V, query: &Self::Query, full_data: &QuiltMetaData) -> Result<Self::Data> {
        let result = match query {
            QuiltQuery::MainClass => VersionMetaData::MainClass(extract_main_class(full_data)),
            QuiltQuery::Libraries => VersionMetaData::Libraries(extract_libraries(full_data).await?),
            QuiltQuery::Arguments => VersionMetaData::Arguments(extract_arguments(full_data)),
            QuiltQuery::QuiltBuilder => VersionMetaData::Version(Self::version_builder(version, full_data).await?),
        };

        Ok(result)
    }

    async fn version_builder<V: VersionInfo>(version: &V, full_data: &QuiltMetaData) -> Result<Version> {
        let (vanilla_builder, quilt_libraries) = tokio::try_join!(
        async {
            let vanilla_data = VanillaQuery::fetch_full_data(version).await?;
            VanillaQuery::version_builder(version, &vanilla_data).await
        },
        extract_libraries(full_data)
    )?;

        Ok(Version {
            main_class: merge_main_class(vanilla_builder.main_class, extract_main_class(full_data)),
            java_version: vanilla_builder.java_version,
            arguments: merge_arguments(vanilla_builder.arguments, extract_arguments(full_data)),
            libraries: merge_libraries(vanilla_builder.libraries, quilt_libraries),
            mods: None,
            natives: vanilla_builder.natives,
            client: vanilla_builder.client,
            assets_index: vanilla_builder.assets_index,
            assets: vanilla_builder.assets,
        })
    }
}

fn merge_main_class(vanilla: MainClass, quilt: MainClass) -> MainClass {
    if quilt.main_class.is_empty() {
        vanilla
    } else {
        quilt
    }
}

fn merge_arguments(vanilla: Arguments, quilt: Arguments) -> Arguments {
    Arguments {
        game: {
            let mut args = vanilla.game;
            args.extend(quilt.game);
            args
        },
        jvm: match (vanilla.jvm, quilt.jvm) {
            (Some(mut v), Some(q)) => {
                v.extend(q);
                Some(v)
            }
            (Some(v), None) => Some(v),
            (None, Some(q)) => Some(q),
            (None, None) => None,
        },
    }
}

/// Merges library lists, de-duplicating by `group:artifact`. Quilt wins.
fn merge_libraries(vanilla_libs: Vec<Library>, quilt_libs: Vec<Library>) -> Vec<Library> {
    let capacity = vanilla_libs.len() + quilt_libs.len();
    let mut lib_map: HashMap<String, Library> = HashMap::with_capacity(capacity);

    for lib in vanilla_libs {
        let key = extract_artifact_key(&lib.name);
        lib_map.insert(key, lib);
    }

    for lib in quilt_libs {
        let key = extract_artifact_key(&lib.name);
        lib_map.insert(key, lib);
    }

    lib_map.into_values().collect()
}

fn extract_artifact_key(maven_name: &str) -> String {
    let mut parts = maven_name.split(':');
    match (parts.next(), parts.next()) {
        (Some(group), Some(artifact)) => format!("{}:{}", group, artifact),
        _ => maven_name.to_string(),
    }
}

fn extract_main_class(full_data: &QuiltMetaData) -> MainClass {
    MainClass {
        main_class: full_data.main_class.clone(),
    }
}


async fn extract_libraries(full_data: &QuiltMetaData) -> Result<Vec<Library>> {
    let futures = full_data.libraries.iter().map(|lib| {
        let lib_name = lib.name.clone();
        let lib_url = lib.url.clone();

        async move {
            let (path, full_url) = maven_artifact_to_path_and_url(&lib_name, &lib_url);

            let (sha1, size) = tokio::join!(
                fetch_maven_sha1(&full_url),
                fetch_file_size(&full_url)
            );

            Library {
                name: lib_name,
                url: Some(full_url),
                path: Some(path),
                sha1,
                size,
            }
        }
    });

    Ok(join_all(futures).await)
}

fn maven_artifact_to_path_and_url(maven_name: &str, base_url: &str) -> (String, String) {
    let mut parts = maven_name.split(':');

    let (group_id, artifact_id, version) = match (parts.next(), parts.next(), parts.next()) {
        (Some(g), Some(a), Some(v)) => (g, a, v),
        _ => return (String::new(), String::new()),
    };

    let group_path = group_id.replace('.', "/");
    let jar_name = format!("{}-{}.jar", artifact_id, version);
    let path = format!("{}/{}/{}/{}", group_path, artifact_id, version, jar_name);
    let base = base_url.trim_end_matches('/');
    let full_url = format!("{}/{}", base, path);

    (path, full_url)
}

fn extract_arguments(full_data: &QuiltMetaData) -> Arguments {
    Arguments {
        game: full_data.arguments.game.clone(),
        jvm: None,
    }
}