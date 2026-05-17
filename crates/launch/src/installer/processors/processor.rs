// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Install-processor executor for Forge (1.13+) and NeoForge.

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

use lighty_core::download::download_file_untracked;
use lighty_core::mkdir;
use lighty_loaders::types::VersionInfo;
use lighty_core::QueryError;
use lighty_loaders::utils::forge_installer::{ForgeInstallProfile, Processor};

type Result<T> = std::result::Result<T, QueryError>;

/// Execution context shared by every install-processor invocation.
pub(crate) struct ProcessorContext {
    pub game_dir: PathBuf,
    pub libraries_dir: PathBuf,
    pub minecraft_version: String,
    pub installer_path: PathBuf,
    pub data: HashMap<String, String>,
    pub side: String,
    pub maven_base_url: String,
    pub extract_subdir: String,
    pub java_path: PathBuf,
}

impl ProcessorContext {
    /// Builds a fresh processor context from an instance and its installer.
    pub fn new<V: VersionInfo>(
        version: &V,
        installer_path: PathBuf,
        metadata: &ForgeInstallProfile,
        maven_base_url: impl Into<String>,
        extract_subdir: impl Into<String>,
        java_path: PathBuf,
    ) -> Self {
        let side = "client".to_string();
        let game_dir = version.game_dirs();
        let libraries_dir = game_dir.join("libraries");

        let mut data = HashMap::new();
        for (key, value) in &metadata.data {
            data.insert(key.clone(), value.client.clone());
        }

        data.insert("ROOT".to_string(), game_dir.to_string_lossy().to_string());
        data.insert(
            "LIBRARY_DIR".to_string(),
            libraries_dir.to_string_lossy().to_string(),
        );
        data.insert(
            "MINECRAFT_VERSION".to_string(),
            version.minecraft_version().to_string(),
        );
        data.insert("SIDE".to_string(), side.clone());
        data.insert(
            "INSTALLER".to_string(),
            installer_path.to_string_lossy().to_string(),
        );

        let minecraft_jar = game_dir.join(format!("{}.jar", version.name()));
        data.insert(
            "MINECRAFT_JAR".to_string(),
            minecraft_jar.to_string_lossy().to_string(),
        );

        Self {
            game_dir: game_dir.to_path_buf(),
            libraries_dir,
            minecraft_version: version.minecraft_version().to_string(),
            installer_path,
            data,
            side,
            maven_base_url: maven_base_url.into(),
            extract_subdir: extract_subdir.into(),
            java_path,
        }
    }

    /// Substitutes only the `{KEY}` placeholders; leaves Maven coordinates
    /// (`[group:artifact:...]`) and installer paths (`/path`) untouched.
    pub fn substitute_variables(&self, input: &str) -> String {
        let mut result = input.to_string();
        for (key, value) in &self.data {
            let pattern = format!("{{{}}}", key);
            result = result.replace(&pattern, value);
        }
        result
    }

    /// Fully resolves a processor argument into a usable filesystem path.
    ///
    /// Handles three forms:
    /// - `{VAR}` placeholders are substituted from [`Self::data`]
    /// - `[group:artifact:version:classifier@extension]` resolves to the
    ///   corresponding path under [`Self::libraries_dir`]
    /// - `/path` is treated as a path inside the installer JAR
    pub fn substitute(&self, input: &str) -> Result<String> {
        let result = self.substitute_variables(input);

        if result.starts_with('[') && result.ends_with(']') {
            let maven_coords = &result[1..result.len() - 1];
            return self.resolve_maven_path(maven_coords);
        }

        if result.starts_with('/') {
            return Ok(result);
        }

        Ok(result)
    }

    /// Resolves Maven coordinates into a filesystem path under
    /// [`Self::libraries_dir`].
    fn resolve_maven_path(&self, maven_coords: &str) -> Result<String> {
        let parts: Vec<&str> = maven_coords.split(':').collect();
        if parts.len() < 3 {
            return Err(QueryError::Conversion {
                message: format!("Invalid Maven coordinate: {}", maven_coords),
            });
        }

        let group = parts[0].replace('.', "/");
        let artifact = parts[1];

        let (version, classifier, extension) = if parts.len() >= 4 {
            let version = parts[2];
            let last_part = parts[3];
            if let Some((clf, ext)) = last_part.split_once('@') {
                (version, Some(clf), ext)
            } else {
                (version, Some(last_part), "jar")
            }
        } else {
            let version_part = parts[2];
            if let Some((ver, ext)) = version_part.split_once('@') {
                (ver, None, ext)
            } else {
                (version_part, None, "jar")
            }
        };

        let filename = if let Some(clf) = classifier {
            format!("{}-{}-{}.{}", artifact, version, clf, extension)
        } else {
            format!("{}-{}.{}", artifact, version, extension)
        };

        let path = self
            .libraries_dir
            .join(&group)
            .join(artifact)
            .join(version)
            .join(&filename);

        Ok(path.to_string_lossy().to_string())
    }

    /// Builds a Maven download URL for the given coordinates on this
    /// context's configured [`maven_base_url`](Self::maven_base_url).
    pub fn build_maven_url(&self, maven_coords: &str) -> Result<String> {
        let parts: Vec<&str> = maven_coords.split(':').collect();
        if parts.len() < 3 {
            return Err(QueryError::Conversion {
                message: format!("Invalid Maven coordinate: {}", maven_coords),
            });
        }

        let group = parts[0].replace('.', "/");
        let artifact = parts[1];

        let (version, classifier, extension) = if parts.len() >= 4 {
            let version = parts[2];
            let last_part = parts[3];
            if let Some((clf, ext)) = last_part.split_once('@') {
                (version, Some(clf), ext)
            } else {
                (version, Some(last_part), "jar")
            }
        } else {
            let version_part = parts[2];
            if let Some((ver, ext)) = version_part.split_once('@') {
                (ver, None, ext)
            } else {
                (version_part, None, "jar")
            }
        };

        let filename = if let Some(clf) = classifier {
            format!("{}-{}-{}.{}", artifact, version, clf, extension)
        } else {
            format!("{}-{}.{}", artifact, version, extension)
        };

        let base_url = self.maven_base_url.trim_end_matches('/');
        Ok(format!(
            "{}/{}/{}/{}/{}",
            base_url, group, artifact, version, filename
        ))
    }

    /// Extracts a single entry from the installer JAR to `output_path`.
    pub async fn extract_installer_file(
        &self,
        internal_path: &str,
        output_path: &Path,
    ) -> Result<()> {
        let internal_path = internal_path.trim_start_matches('/');

        let file = File::open(&self.installer_path).map_err(|e| QueryError::Conversion {
            message: format!("Failed to open installer JAR: {}", e),
        })?;

        let mut archive = ZipArchive::new(file).map_err(|e| QueryError::Conversion {
            message: format!("Failed to open ZIP archive: {}", e),
        })?;

        let mut zip_file =
            archive
                .by_name(internal_path)
                .map_err(|_| QueryError::MissingField {
                    field: format!("{} in installer JAR", internal_path),
                })?;

        if let Some(parent) = output_path.parent() {
            mkdir!(parent);
        }

        let mut output = File::create(output_path).map_err(|e| QueryError::Conversion {
            message: format!("Failed to create output file: {}", e),
        })?;

        std::io::copy(&mut zip_file, &mut output).map_err(|e| QueryError::Conversion {
            message: format!("Failed to extract file: {}", e),
        })?;

        Ok(())
    }
}

/// Extracts every `maven/...` entry from the installer JAR into `libraries_dir`.
///
/// Forge installers ship some artifacts (forge-shim.jar in 1.21+, forge.jar
/// in 1.14) bundled at `/maven/...` instead of publishing them. Idempotent:
/// skips entries whose target already exists with the right size. NeoForge
/// installers carry no `/maven/` entries, so this is a no-op for them.
pub(crate) fn extract_maven_bundle_to_libraries(
    installer_path: &Path,
    libraries_dir: &Path,
) -> Result<()> {
    let file = File::open(installer_path).map_err(|e| QueryError::Conversion {
        message: format!("Failed to open installer JAR: {}", e),
    })?;

    let mut archive = ZipArchive::new(file).map_err(|e| QueryError::Conversion {
        message: format!("Failed to open ZIP archive: {}", e),
    })?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| QueryError::Conversion {
            message: format!("Failed to read ZIP entry {}: {}", i, e),
        })?;

        if entry.is_dir() {
            continue;
        }
        let Some(rel) = entry.enclosed_name() else { continue };
        let Ok(rel_path) = rel.strip_prefix("maven/") else { continue };

        let dest = libraries_dir.join(rel_path);

        if let Ok(meta) = std::fs::metadata(&dest) {
            if meta.len() == entry.size() {
                continue;
            }
        }

        if let Some(parent) = dest.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| QueryError::Conversion {
                    message: format!("Failed to create dir {}: {}", parent.display(), e),
                })?;
            }
        }

        let mut out = File::create(&dest).map_err(|e| QueryError::Conversion {
            message: format!("Failed to create {}: {}", dest.display(), e),
        })?;
        std::io::copy(&mut entry, &mut out).map_err(|e| QueryError::Conversion {
            message: format!("Failed to extract {}: {}", rel_path.display(), e),
        })?;

        lighty_core::trace_debug!(
            path = %dest.display(),
            "Extracted bundled installer artifact"
        );
    }

    Ok(())
}

/// Runs every processor whose `sides` list matches the current side
/// (defaults to `"client"`).
pub(crate) async fn run_processors<V: VersionInfo>(
    version: &V,
    metadata: &ForgeInstallProfile,
    installer_path: PathBuf,
    maven_base_url: impl Into<String>,
    extract_subdir: impl Into<String>,
    java_path: PathBuf,
) -> Result<()> {
    let context = ProcessorContext::new(
        version,
        installer_path,
        metadata,
        maven_base_url,
        extract_subdir,
        java_path,
    );

    lighty_core::trace_info!("Starting processor execution");

    let total_processors = metadata.processors.len();

    let processors: Vec<&Processor> = metadata
        .processors
        .iter()
        .filter(|p| {
            let should_execute = p.sides.is_empty() || p.sides.contains(&context.side);
            if !should_execute {
                lighty_core::trace_debug!(
                    jar = %p.jar,
                    sides = ?p.sides,
                    "Skipping processor (not for client side)"
                );
            }
            should_execute
        })
        .collect();

    let _skipped_count = total_processors - processors.len();

    lighty_core::trace_info!(
        total = total_processors,
        client_processors = processors.len(),
        skipped = _skipped_count,
        "Filtered processors for client side"
    );

    for (_idx, processor) in processors.iter().enumerate() {
        lighty_core::trace_info!(
            processor_num = _idx + 1,
            total = processors.len(),
            jar = %processor.jar,
            "Executing processor"
        );
        execute_processor(&context, processor).await?;
    }

    lighty_core::trace_info!("All processors completed successfully");
    Ok(())
}

/// Runs a single processor.
async fn execute_processor(context: &ProcessorContext, processor: &Processor) -> Result<()> {
    if should_skip_processor(context, processor)? {
        lighty_core::trace_info!("Processor outputs already exist, skipping");
        return Ok(());
    }

    let jar_path = download_processor_jar(context, &processor.jar).await?;
    let mut classpath_paths = vec![jar_path.clone()];

    for cp in &processor.classpath {
        let cp_path = download_processor_jar(context, cp).await?;
        classpath_paths.push(cp_path);
    }

    // Detection runs on the substituted value because `{KEY}` placeholders
    // can resolve to either a Maven coord or an installer-internal path.
    let mut processed_args = Vec::new();
    for arg in &processor.args {
        let substituted = context.substitute_variables(arg);

        if substituted.starts_with('[') && substituted.ends_with(']') {
            let maven_coords = &substituted[1..substituted.len() - 1];
            let resolved_path = context.resolve_maven_path(maven_coords)?;
            let path = PathBuf::from(&resolved_path);
            if !path.exists() {
                // A 404 means the artifact is a processor output produced later.
                match download_processor_jar(context, maven_coords).await {
                    Ok(_) => {}
                    Err(_) => {
                        if let Some(parent) = path.parent() {
                            mkdir!(parent);
                        }
                        lighty_core::trace_debug!(
                            artifact = %maven_coords,
                            "Artifact not available on Maven, assuming processor output"
                        );
                    }
                }
            }
            processed_args.push(resolved_path);
        } else if substituted.starts_with('/') {
            let internal_path = &substituted[1..];
            match extract_and_resolve(context, internal_path).await {
                Ok(target_path) => processed_args.push(target_path),
                Err(_) => {
                    processed_args.push(substituted);
                }
            }
        } else {
            processed_args.push(substituted);
        }
    }

    let main_class = extract_main_class(&jar_path)?;

    let classpath_strings: Vec<String> = classpath_paths
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    let classpath = classpath_strings.join(if cfg!(windows) { ";" } else { ":" });

    lighty_core::trace_debug!(
        main_class = %main_class,
        classpath_count = classpath_paths.len(),
        args_count = processed_args.len(),
        "Processor configuration prepared"
    );

    let output = tokio::process::Command::new(&context.java_path)
        .arg("-cp")
        .arg(&classpath)
        .arg(&main_class)
        .args(&processed_args)
        .current_dir(&context.game_dir)
        .output()
        .await
        .map_err(|e| QueryError::Conversion {
            message: format!("Failed to execute processor: {}", e),
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(QueryError::Conversion {
            message: format!(
                "Processor failed:\nSTDOUT:\n{}\nSTDERR:\n{}",
                stdout, stderr
            ),
        });
    }

    lighty_core::trace_debug!("Processor completed successfully");
    Ok(())
}

/// Downloads a processor JAR from its Maven coordinates and returns the
/// local path to the cached file.
async fn download_processor_jar(
    context: &ProcessorContext,
    maven_coords: &str,
) -> Result<PathBuf> {
    let file_path = context.resolve_maven_path(maven_coords)?;
    let path = PathBuf::from(&file_path);

    if path.exists() {
        return Ok(path);
    }

    let url = context.build_maven_url(maven_coords)?;

    lighty_core::trace_debug!(
        artifact = %maven_coords,
        "Downloading processor dependency"
    );

    if let Some(parent) = path.parent() {
        mkdir!(parent);
    }

    download_file_untracked(&url, &path)
        .await
        .map_err(|e| QueryError::Conversion {
            message: format!("Failed to download {}: {}", maven_coords, e),
        })?;

    Ok(path)
}

/// Extracts an entry from the installer JAR and returns the path to the
/// extracted file under the launcher's libraries layout.
async fn extract_and_resolve(context: &ProcessorContext, internal_path: &str) -> Result<String> {
    let file_name = internal_path
        .split('/')
        .last()
        .ok_or_else(|| QueryError::Conversion {
            message: format!("Invalid internal path: {}", internal_path),
        })?;

    let mut output_path = context.libraries_dir.clone();
    for segment in context.extract_subdir.split('/').filter(|s| !s.is_empty()) {
        output_path.push(segment);
    }
    output_path = output_path
        .join("installer-extracts")
        .join(&context.minecraft_version)
        .join(file_name);

    if !output_path.exists() {
        context
            .extract_installer_file(internal_path, &output_path)
            .await?;
    }

    Ok(output_path.to_string_lossy().to_string())
}

/// Returns `true` when every declared output already exists with the
/// expected hash, meaning the processor can be skipped.
fn should_skip_processor(context: &ProcessorContext, processor: &Processor) -> Result<bool> {
    if processor.outputs.is_empty() {
        return Ok(false);
    }

    for (output_path_pattern, expected_hash_pattern) in &processor.outputs {
        let output_path = context.substitute(output_path_pattern)?;
        let expected_hash = context.substitute(expected_hash_pattern)?;

        let expected_hash = expected_hash.trim_matches('\'').trim_matches('"');
        let path = PathBuf::from(&output_path);

        if !path.exists() {
            return Ok(false);
        }

        match lighty_core::verify_file_sha1_sync(&path, expected_hash) {
            Ok(true) => continue,
            _ => return Ok(false),
        }
    }

    Ok(true)
}

/// Reads the `Main-Class` entry from a JAR's `META-INF/MANIFEST.MF`.
fn extract_main_class(jar_path: &Path) -> Result<String> {
    let file = File::open(jar_path).map_err(|e| QueryError::Conversion {
        message: format!("Failed to open JAR: {}", e),
    })?;

    let mut archive = ZipArchive::new(file).map_err(|e| QueryError::Conversion {
        message: format!("Failed to open ZIP archive: {}", e),
    })?;

    let mut manifest_file =
        archive
            .by_name("META-INF/MANIFEST.MF")
            .map_err(|_| QueryError::MissingField {
                field: "META-INF/MANIFEST.MF in processor JAR".to_string(),
            })?;

    let mut contents = String::new();
    manifest_file
        .read_to_string(&mut contents)
        .map_err(|e| QueryError::Conversion {
            message: format!("Failed to read MANIFEST.MF: {}", e),
        })?;

    for line in contents.lines() {
        if let Some(main_class) = line.strip_prefix("Main-Class:") {
            return Ok(main_class.trim().to_string());
        }
    }

    Err(QueryError::MissingField {
        field: "Main-Class in MANIFEST.MF".to_string(),
    })
}
