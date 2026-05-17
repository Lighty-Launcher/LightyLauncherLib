// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Legacy Forge (Minecraft 1.4.x -> 1.12.2): single `install_profile.json`
//! with embedded `versionInfo`, no processors, universal JAR extracted
//! from the installer ZIP.

use std::{collections::HashMap, fs::File, io::Read, path::{Path, PathBuf}};

use futures::future::join_all;
use zip::ZipArchive;

use lighty_core::download::download_file_untracked;
use lighty_core::mkdir;

use lighty_core::system::OS;

use crate::loaders::vanilla::vanilla::{should_apply_rules, VanillaQuery};
use crate::types::version_metadata::{Arguments, Library, MainClass, Version};
use crate::types::VersionInfo;
use lighty_core::QueryError;
use crate::utils::maven::probe_maven_bases;
use crate::utils::query::Query;

use super::forge_legacy_metadata::{ForgeLegacyInstallProfile, ForgeLegacyLibrary};

pub type Result<T> = std::result::Result<T, QueryError>;

/// Discriminates which install_profile.json schema an installer ships.
///
/// Used by the Forge dispatcher when the Minecraft version alone is
/// ambiguous (e.g. Forge 14.23.5.2860 for MC 1.12.2 is a back-ported
/// build that uses the modern schema).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallProfileKind {
    Modern,
    Legacy,
}

const FORGE_MAVEN: &str = "https://maven.minecraftforge.net";

/// Probe order for legacy library entries shipped without an explicit `url`.
/// Same list the original Forge installer uses: Mojang libs (vanilla shared
/// artifacts), Forge Maven (Forge-shipped), then Maven Central as fallback.
const LEGACY_FALLBACK_BASES: [&str; 3] = [
    "https://libraries.minecraft.net/",
    "https://maven.minecraftforge.net/",
    "https://repo.maven.apache.org/maven2/",
];

/// Returns whether the Minecraft version predates the modern Forge
/// installer format (which started at 1.13).
pub fn is_legacy_forge(mc: &str) -> bool {
    version_compare::compare_to(mc, "1.13", version_compare::Cmp::Lt).unwrap_or(false)
}

/// Returns whether the Minecraft version predates Forge's installer
/// format (<= 1.5.1, only `-universal.zip` artifacts exist).
fn is_pre_installer_forge(mc: &str) -> bool {
    version_compare::compare_to(mc, "1.5.2", version_compare::Cmp::Lt).unwrap_or(false)
}

/// Builds the two candidate installer URLs for a legacy Forge build:
/// single-suffix (`{mc}-{fg}/forge-{mc}-{fg}-installer.jar`, used by most
/// builds) and double-suffix (used by canonical 1.7.10 and some early
/// 1.7.2). Both shapes coexist across 1.5-1.7; the caller HEAD-probes.
fn legacy_installer_url_candidates<V: VersionInfo>(version: &V) -> (String, String) {
    let mc = version.minecraft_version();
    let fg = version.loader_version();
    let single = format!(
        "{}/net/minecraftforge/forge/{}-{}/forge-{}-{}-installer.jar",
        FORGE_MAVEN, mc, fg, mc, fg
    );
    let double = format!(
        "{}/net/minecraftforge/forge/{}-{}-{}/forge-{}-{}-{}-installer.jar",
        FORGE_MAVEN, mc, fg, mc, mc, fg, mc
    );
    (single, double)
}

/// Probes both candidate installer URLs and returns the one that exists.
async fn resolve_legacy_installer_url<V: VersionInfo>(version: &V) -> Option<String> {
    let (single, double) = legacy_installer_url_candidates(version);
    for candidate in [&single, &double] {
        let (base, file) = match candidate.rsplit_once('/') {
            Some((b, f)) => (format!("{}/", b), f),
            None => continue,
        };
        if let Some(url) = probe_maven_bases(&[base.as_str()], file).await {
            return Some(url);
        }
    }
    None
}

/// Path to the cached installer JAR. Same naming as modern Forge so
/// both dispatchers find the same on-disk file.
pub fn legacy_installer_path<V: VersionInfo>(version: &V) -> PathBuf {
    version
        .game_dirs()
        .join(".forge")
        .join(format!("forge-{}-installer.jar", version.loader_version()))
}

/// Downloads (or reuses) the legacy installer JAR for `version` and
/// returns its on-disk path.
pub async fn ensure_installer_cached<V: VersionInfo>(version: &V) -> Result<PathBuf> {
    let mc = version.minecraft_version();
    if is_pre_installer_forge(mc) {
        return Err(QueryError::UnsupportedLoader(format!(
            "Forge for Minecraft {} predates the installer format (1.5.2+ only)",
            mc
        )));
    }

    let profiles_dir = version.game_dirs().join(".forge");
    mkdir!(profiles_dir);

    let installer_path = legacy_installer_path(version);

    if !installer_path.exists() {
        let installer_url =
            resolve_legacy_installer_url(version)
                .await
                .ok_or_else(|| QueryError::Conversion {
                    message: format!(
                        "No installer JAR found on the Forge Maven for {}-{} (tried single \
                         and double MC-suffix layouts)",
                        version.minecraft_version(),
                        version.loader_version()
                    ),
                })?;
        lighty_core::trace_debug!(url = %installer_url, loader = "forge", "Legacy installer URL");
        lighty_core::trace_info!(
            loader = "forge",
            path = ?installer_path,
            "Downloading legacy installer"
        );
        download_file_untracked(&installer_url, &installer_path)
            .await
            .map_err(|e| QueryError::Conversion {
                message: format!("Failed to download legacy installer: {}", e),
            })?;
    }

    Ok(installer_path)
}

/// Detects whether an installer JAR ships a legacy or modern
/// `install_profile.json` by inspecting its top-level keys.
///
/// `versionInfo` => legacy; anything else => modern. Some 1.12.2 builds
/// (e.g. 14.23.5.2860) were repackaged with the modern schema even
/// though MC < 1.13, so dispatching on MC version alone is wrong.
pub async fn peek_install_profile_kind(installer_path: &Path) -> Result<InstallProfileKind> {
    let path = installer_path.to_owned();
    tokio::task::spawn_blocking(move || {
        let file = File::open(&path).map_err(|e| QueryError::Conversion {
            message: format!("Failed to open installer JAR: {}", e),
        })?;
        let mut archive = ZipArchive::new(file).map_err(|e| QueryError::Conversion {
            message: format!("Failed to open ZIP archive: {}", e),
        })?;
        let mut profile_file = archive
            .by_name("install_profile.json")
            .map_err(|_| QueryError::MissingField {
                field: "install_profile.json in installer JAR".to_string(),
            })?;
        let mut contents = String::new();
        profile_file
            .read_to_string(&mut contents)
            .map_err(|e| QueryError::Conversion {
                message: format!("Failed to read install_profile.json: {}", e),
            })?;
        let v: serde_json::Value = serde_json::from_str(&contents)?;
        if v.get("versionInfo").is_some() {
            Ok(InstallProfileKind::Legacy)
        } else {
            Ok(InstallProfileKind::Modern)
        }
    })
    .await
    .map_err(|e| QueryError::Conversion {
        message: format!("Failed to spawn blocking task: {}", e),
    })?
}

/// Downloads the legacy installer JAR (cached) and reads
/// `install_profile.json` from inside it.
pub async fn fetch_legacy_install_profile<V: VersionInfo>(
    version: &V,
) -> Result<ForgeLegacyInstallProfile> {
    let installer_path = ensure_installer_cached(version).await?;
    read_install_profile_from_jar(&installer_path).await
}

/// Reads `install_profile.json` directly from the installer JAR.
pub async fn read_install_profile_from_jar(
    installer_path: &Path,
) -> Result<ForgeLegacyInstallProfile> {
    let path = installer_path.to_owned();
    tokio::task::spawn_blocking(move || {
        let file = File::open(&path).map_err(|e| QueryError::Conversion {
            message: format!("Failed to open installer JAR: {}", e),
        })?;
        let mut archive = ZipArchive::new(file).map_err(|e| QueryError::Conversion {
            message: format!("Failed to open ZIP archive: {}", e),
        })?;

        let mut profile_file =
            archive
                .by_name("install_profile.json")
                .map_err(|_| QueryError::MissingField {
                    field: "install_profile.json in installer JAR".to_string(),
                })?;

        let mut contents = String::new();
        profile_file
            .read_to_string(&mut contents)
            .map_err(|e| QueryError::Conversion {
                message: format!("Failed to read install_profile.json: {}", e),
            })?;

        serde_json::from_str::<ForgeLegacyInstallProfile>(&contents).map_err(QueryError::from)
    })
    .await
    .map_err(|e| QueryError::Conversion {
        message: format!("Failed to spawn blocking task: {}", e),
    })?
}

/// Returns the relative path under `libraries/` for a Maven coordinate.
fn maven_relative_path(name: &str) -> Option<String> {
    let mut parts = name.split(':');
    let group = parts.next()?;
    let artifact = parts.next()?;
    let version = parts.next()?;
    let group_path = group.replace('.', "/");
    let filename = format!("{}-{}.jar", artifact, version);
    Some(format!("{}/{}/{}/{}", group_path, artifact, version, filename))
}

/// Returns whether `lib_name` is the Forge universal JAR declared by
/// `profile.install.path`. The artifact name shifts across the legacy
/// era (1.6.x: `minecraftforge`, 1.7+: `forge`), so we match the
/// install profile's declaration rather than hardcoding a prefix.
fn is_universal_artifact(lib_name: &str, install_path: &str) -> bool {
    fn group_artifact(coord: &str) -> Option<(&str, &str)> {
        let mut parts = coord.split(':');
        let group = parts.next()?;
        let artifact = parts.next()?;
        Some((group, artifact))
    }
    match (group_artifact(lib_name), group_artifact(install_path)) {
        (Some(lib_coord), Some(universal_coord)) => lib_coord == universal_coord,
        _ => false,
    }
}

/// Normalizes legacy library base URLs (http -> https, trailing slash).
fn normalize_lib_base(url: &str) -> String {
    let mut s = url.to_string();
    s = s.replace(
        "http://files.minecraftforge.net/maven/",
        "https://maven.minecraftforge.net/",
    );
    s = s.replace(
        "http://libraries.minecraft.net/",
        "https://libraries.minecraft.net/",
    );
    if !s.ends_with('/') {
        s.push('/');
    }
    s
}

/// Resolves a legacy library entry to its final download URL.
///
/// Entries with an explicit `url` use that base. Entries without `url`
/// are HEAD-probed against [`LEGACY_FALLBACK_BASES`]; the first base
/// serving a non-empty body wins. When every probe fails we fall back
/// to Mojang libs so the downloader surfaces a clear 404.
async fn resolve_legacy_url(lib: &ForgeLegacyLibrary, relative_path: &str) -> String {
    if let Some(base) = lib.url.as_deref() {
        return format!("{}{}", normalize_lib_base(base), relative_path);
    }
    if let Some(url) = probe_maven_bases(&LEGACY_FALLBACK_BASES, relative_path).await {
        return url;
    }
    // Emit a Mojang-libs URL so the downloader fails loudly with a 404
    // rather than silently leaving a classpath gap.
    format!("{}{}", LEGACY_FALLBACK_BASES[0], relative_path)
}

/// Converts a legacy library entry into the launcher's pivot `Library`.
///
/// Returns `None` for entries the launcher should skip (server-only).
/// The Forge universal JAR is included with `url: None` so the
/// installer pipeline doesn't try to fetch it from Maven — it is
/// extracted from the installer by [`extract_universal_jar`] instead.
async fn legacy_library_to_pivot(
    lib: &ForgeLegacyLibrary,
    install_path: &str,
) -> Option<Library> {
    if !lib.clientreq {
        return None;
    }

    // Skip libs explicitly disallowed for the current platform
    // (1.6.x bundles mac-only LWJGL 2.9.1-nightly variants).
    if let Some(rules) = &lib.rules {
        let os_name = OS.get_vanilla_os().ok()?;
        if !should_apply_rules(rules, os_name) {
            return None;
        }
    }

    // Mojang's CDN dropped the bare-JAR side of natives-only entries in
    // the 1.5/1.6 era, only `-natives-{os}` classifiers remain. The
    // vanilla natives extractor handles them; the libraries pipeline
    // must skip them to avoid fetching a non-existent base JAR.
    if lib.natives.is_some() {
        return None;
    }

    let path = maven_relative_path(&lib.name)?;

    if is_universal_artifact(&lib.name, install_path) {
        return Some(Library {
            name: lib.name.clone(),
            url: None,
            path: Some(path),
            sha1: None,
            size: None,
        });
    }

    let full_url = resolve_legacy_url(lib, &path).await;

    Some(Library {
        name: lib.name.clone(),
        url: Some(full_url),
        path: Some(path),
        sha1: None,
        size: None,
    })
}

/// Builds the runtime library list from a legacy install profile,
/// filtering server-only entries and resolving Maven URLs (probes
/// performed in parallel).
pub async fn extract_legacy_libraries(profile: &ForgeLegacyInstallProfile) -> Vec<Library> {
    let install_path = profile.install.path.as_str();
    let futures = profile
        .version_info
        .libraries
        .iter()
        .map(|lib| legacy_library_to_pivot(lib, install_path));
    join_all(futures).await.into_iter().flatten().collect()
}

/// Parses a legacy `minecraftArguments` string into the pivot `Arguments`,
/// appending `extra_jvm` to the JVM flag list when non-empty.
fn parse_legacy_arguments(s: &str, extra_jvm: Vec<String>) -> Arguments {
    Arguments {
        game: s.split_whitespace().map(String::from).collect(),
        jvm: if extra_jvm.is_empty() { None } else { Some(extra_jvm) },
    }
}

/// Returns JVM flags that legacy FML needs for `mc`.
///
/// FML 1.6.x rejects client JARs whose filename isn't `minecraft.jar` or
/// the `versions/X.X.X/X.X.X.jar` layout. We stage the client JAR as
/// `<instance>.jar` so FML aborts with "CRITICAL TAMPERING" unless we set
/// the bypass flag. 1.7+ downgrades this to a warning.
fn legacy_fml_jvm_workarounds(mc: &str) -> Vec<String> {
    if version_compare::compare_to(mc, "1.7", version_compare::Cmp::Lt).unwrap_or(false) {
        vec!["-Dfml.ignoreInvalidMinecraftCertificates=true".to_string()]
    } else {
        Vec::new()
    }
}

/// Merges library lists, de-duplicating by `group:artifact`. Forge wins.
fn merge_libraries(vanilla_libs: Vec<Library>, forge_libs: Vec<Library>) -> Vec<Library> {
    let capacity = vanilla_libs.len() + forge_libs.len();
    let mut lib_map: HashMap<String, Library> = HashMap::with_capacity(capacity);
    for lib in vanilla_libs {
        lib_map.insert(extract_artifact_key(&lib.name), lib);
    }
    for lib in forge_libs {
        lib_map.insert(extract_artifact_key(&lib.name), lib);
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

/// Builds the full pivot `Version` for a legacy Forge instance.
pub async fn legacy_version_builder<V: VersionInfo>(
    version: &V,
    profile: &ForgeLegacyInstallProfile,
) -> Result<Version> {
    let vanilla_data = VanillaQuery::fetch_full_data(version).await?;
    let vanilla_builder = VanillaQuery::version_builder(version, &vanilla_data).await?;

    let forge_libs = extract_legacy_libraries(profile).await;
    let merged_libs = merge_libraries(vanilla_builder.libraries, forge_libs);

    let extra_jvm = legacy_fml_jvm_workarounds(version.minecraft_version());

    Ok(Version {
        main_class: MainClass {
            main_class: profile.version_info.main_class.clone(),
        },
        java_version: vanilla_builder.java_version,
        arguments: parse_legacy_arguments(
            &profile.version_info.minecraft_arguments,
            extra_jvm,
        ),
        libraries: merged_libs,
        mods: None,
        natives: vanilla_builder.natives,
        client: vanilla_builder.client,
        assets_index: vanilla_builder.assets_index,
        assets: vanilla_builder.assets,
    })
}

/// Extracts the universal JAR from the cached legacy installer to its
/// target Maven path under `{game_dir}/libraries/`.
pub async fn extract_universal_jar<V: VersionInfo>(
    version: &V,
    profile: &ForgeLegacyInstallProfile,
) -> Result<()> {
    let installer_path = legacy_installer_path(version);

    if !installer_path.exists() {
        return Err(QueryError::Conversion {
            message: "Legacy installer JAR not on disk; fetch_legacy_install_profile must run first"
                .to_string(),
        });
    }

    let target_path = version.game_dirs().join("libraries").join(
        maven_relative_path(&profile.install.path).ok_or_else(|| QueryError::Conversion {
            message: format!("Invalid Maven coord in install.path: {}", profile.install.path),
        })?,
    );

    if target_path.exists() {
        return Ok(());
    }

    if let Some(parent) = target_path.parent() {
        mkdir!(parent);
    }

    let installer = installer_path.clone();
    let target = target_path.clone();
    let internal_name = profile.install.file_path.clone();

    tokio::task::spawn_blocking(move || {
        let file = File::open(&installer).map_err(|e| QueryError::Conversion {
            message: format!("Failed to open installer JAR: {}", e),
        })?;
        let mut archive = ZipArchive::new(file).map_err(|e| QueryError::Conversion {
            message: format!("Failed to open ZIP archive: {}", e),
        })?;

        let mut entry = archive
            .by_name(&internal_name)
            .map_err(|_| QueryError::MissingField {
                field: format!("{} in installer JAR", internal_name),
            })?;

        let mut out = File::create(&target).map_err(|e| QueryError::Conversion {
            message: format!("Failed to create universal JAR target file: {}", e),
        })?;

        std::io::copy(&mut entry, &mut out).map_err(|e| QueryError::Conversion {
            message: format!("Failed to extract universal JAR: {}", e),
        })?;

        Ok::<_, QueryError>(())
    })
    .await
    .map_err(|e| QueryError::Conversion {
        message: format!("Failed to spawn blocking task: {}", e),
    })??;

    lighty_core::trace_info!(
        loader = "forge",
        target = %target_path.display(),
        "Extracted legacy Forge universal JAR"
    );

    Ok(())
}
