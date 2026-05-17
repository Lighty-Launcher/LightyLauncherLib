use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use lighty_auth::UserProfile;
use lighty_core::time_it;
#[cfg(feature = "events")]
use lighty_event::EventBus;
use lighty_java::jre_downloader::{find_java_binary, jre_download};
use lighty_java::runtime::JavaRuntime;
use lighty_java::JavaDistribution;
#[cfg(not(feature = "events"))]
use lighty_java::JreError;
use lighty_loaders::types::version_metadata::{Version, VersionMetaData};
use lighty_loaders::types::{Loader, LoaderExtensions, VersionInfo};
use lighty_modsloader::WithMods;

use crate::arguments::Arguments;
use crate::errors::{InstallerError, InstallerResult};
#[cfg(any(feature = "neoforge", feature = "forge"))]
use crate::installer::ressources::libraries::{collect_library_tasks, download_libraries};
use crate::installer::Installer;

use super::builder::LaunchBuilder;

#[cfg(feature = "forge")]
use crate::installer::processors::forge_install::run_forge_install_processors;
#[cfg(feature = "neoforge")]
use crate::installer::processors::forge_install::run_neoforge_install_processors;

#[cfg(feature = "forge")]
use lighty_loaders::forge::forge::{
    extract_install_profile_libraries_modern as forge_install_profile_libraries_modern,
    ForgeRawData, FORGE,
};
#[cfg(feature = "forge")]
use lighty_loaders::forge::forge_legacy::extract_universal_jar as forge_legacy_extract_universal_jar;
#[cfg(feature = "neoforge")]
use lighty_loaders::neoforge::neoforge::{
    extract_install_profile_libraries as neoforge_install_profile_libraries, NEOFORGE,
};

/// Extension trait that adds [`Self::launch`] to any installable instance.
///
/// Implemented automatically for every type that satisfies the launch
/// pipeline's trait bounds (see the blanket impl below).
pub trait Launch {
    /// Launch the game with a builder pattern
    ///
    /// # Arguments
    /// - `profile`: User profile from authentication
    /// - `java_distribution`: Java distribution to use
    ///
    /// # Returns
    /// A `LaunchBuilder` for configuring JVM options and game arguments
    ///
    /// # Example
    /// ```no_run
    /// // Simple launch
    /// version.launch(&profile, JavaDistribution::Zulu).await?;
    ///
    /// // With custom options
    /// version.launch(&profile, JavaDistribution::Zulu)
    ///     .with_jvm_options()
    ///         .set("Xmx", "4G")
    ///         .done()
    ///     .with_arguments()
    ///         .set(KEY_WIDTH, "1920")
    ///         .done()
    ///     .await?;
    /// ```
    fn launch<'a>(
        &'a mut self,
        profile: &'a UserProfile,
        java_distribution: JavaDistribution,
    ) -> LaunchBuilder<'a, Self>
    where
        Self: Sized;
}

// Blanket impl for any type that implements the launch pipeline traits.
//
// `WithMods` is always required (provides `mod_requests()` — defaults to
// `&[]` for types that don't track mods, so it's free for vanilla
// instances). Modrinth / CurseForge / modpack features only gate which
// builder methods exist and which API clients compile in.
impl<T> Launch for T
where
    T: VersionInfo<LoaderType = Loader>
        + LoaderExtensions
        + Arguments
        + Installer
        + WithMods,
{
    fn launch<'a>(
        &'a mut self,
        profile: &'a UserProfile,
        java_distribution: JavaDistribution,
    ) -> LaunchBuilder<'a, Self> {
        LaunchBuilder::new(self, profile, java_distribution)
    }
}

/// Internal function to execute the launch process
pub(crate) async fn execute_launch<T>(
    version: &mut T,
    profile: &UserProfile,
    java_distribution: JavaDistribution,
    jvm_overrides: &std::collections::HashMap<String, String>,
    jvm_removals: &std::collections::HashSet<String>,
    arg_overrides: &std::collections::HashMap<String, String>,
    arg_removals: &std::collections::HashSet<String>,
    raw_args: &[String],
    #[cfg(feature = "events")] event_bus: Option<&EventBus>,
) -> InstallerResult<()>
where
    T: VersionInfo<LoaderType = Loader>
        + LoaderExtensions
        + Arguments
        + Installer
        + WithMods,
{
    // 1. Fetch the loader metadata
    let metadata = prepare_metadata(
        version,
        #[cfg(feature = "events")]
        event_bus,
    )
    .await?;

    let version_data = extract_version(&metadata)?;

    // 2. Make sure Java is installed
    let java_path = ensure_java_installed(
        version,
        version_data,
        &java_distribution,
        #[cfg(feature = "events")]
        event_bus,
    )
    .await?;

    // Reconcile arg_overrides[KEY_GAME_DIRECTORY] back onto the
    // builder so install + args read the same value via
    // version.runtime_dir(). `game_dirs.join(custom)` does what we
    // want: a relative override ("runtime") resolves to
    // game_dirs/runtime, an absolute override ("/mnt/games") wins
    // outright (Path::join semantics).
    if let Some(custom) = arg_overrides.get(crate::arguments::KEY_GAME_DIRECTORY) {
        let resolved = version.game_dirs().join(custom);
        if resolved.as_path() != version.runtime_dir() {
            lighty_core::trace_info!(
                from = %version.runtime_dir().display(),
                to = %resolved.display(),
                source = %custom,
                "[Launch] Resolved KEY_GAME_DIRECTORY override before install"
            );
            version.set_runtime_dir(resolved);
        }
    }

    // Modpack + user-attached mods are resolved by `Installer::install`
    // itself (Phase 0 of its pipeline). The runner just passes the raw
    // `Version` here — no merge to do.

    // Install Minecraft dependencies (libraries, natives, client, assets)
    time_it!(
        "Install delay",
        version
            .install(
                version_data,
                #[cfg(feature = "events")]
                event_bus,
            )
            .await?
    );

    // Forge-family install_profile libraries + processors.
    //
    // For Forge and NeoForge, the install_profile.json libraries are
    // downloaded through the shared library installer (parallel +
    // retry + SHA1) so the processor JARs and the runtime-required
    // `forge:universal` artifact land on disk. Only the processor
    // execution stays inside each loader crate (it's a per-loader
    // Java exec with different maven URLs / extract subdirs).
    //
    // TODO: generalize this into a per-loader post-install hook for any
    // loader that needs one (currently only Forge / NeoForge do).
    #[cfg(feature = "neoforge")]
    if matches!(version.loader(), Loader::NeoForge) {
        let install_profile = NEOFORGE.get_raw(version).await?;
        let profile_libs = neoforge_install_profile_libraries(install_profile.as_ref());
        let profile_tasks = collect_library_tasks(version, &profile_libs).await;
        download_libraries(
            profile_tasks,
            #[cfg(feature = "events")]
            event_bus,
        )
        .await?;
        run_neoforge_install_processors(version, install_profile.as_ref(), java_path.clone())
            .await?;
    }

    #[cfg(feature = "forge")]
    if matches!(version.loader(), Loader::Forge) {
        let raw = FORGE.get_raw(version).await?;
        match raw.as_ref() {
            ForgeRawData::Modern {
                install_profile, ..
            } => {
                // Download processor-only libraries, then run processors.
                let profile_libs = forge_install_profile_libraries_modern(install_profile);
                let profile_tasks = collect_library_tasks(version, &profile_libs).await;
                download_libraries(
                    profile_tasks,
                    #[cfg(feature = "events")]
                    event_bus,
                )
                .await?;
                run_forge_install_processors(version, install_profile, java_path.clone()).await?;
            }
            ForgeRawData::Legacy(profile) => {
                // No processors in the legacy era; the universal JAR
                // ships inside the installer and must be extracted to
                // its Maven path so the classpath entry resolves.
                forge_legacy_extract_universal_jar(version, profile).await?;
            }
        }
    }

    // Launch the game
    execute_game(
        version,
        version_data,
        profile,
        java_path,
        arg_overrides,
        arg_removals,
        jvm_overrides,
        jvm_removals,
        raw_args,
        #[cfg(feature = "events")]
        event_bus,
    )
    .await
}

/// Fetches the loader's full metadata document.
async fn prepare_metadata<T>(
    builder: &mut T,
    #[cfg(feature = "events")] event_bus: Option<&EventBus>,
) -> InstallerResult<Arc<VersionMetaData>>
where
    T: VersionInfo<LoaderType = Loader> + LoaderExtensions,
{
    lighty_core::trace_debug!(
        "[Launch] Fetching metadata for loader: {:?}",
        builder.loader()
    );

    #[cfg(feature = "events")]
    let loader_name = format!("{:?}", builder.loader());

    #[cfg(feature = "events")]
    if let Some(bus) = event_bus {
        bus.emit(lighty_event::Event::Loader(
            lighty_event::LoaderEvent::FetchingData {
                loader: loader_name.clone(),
                minecraft_version: builder.minecraft_version().to_string(),
                loader_version: builder.loader_version().to_string(),
            },
        ));
    }

    // Generic metadata fetching - automatically dispatches to the correct loader
    let metadata = builder.get_metadata().await?;

    #[cfg(feature = "events")]
    if let Some(bus) = event_bus {
        bus.emit(lighty_event::Event::Loader(
            lighty_event::LoaderEvent::DataFetched {
                loader: loader_name,
                minecraft_version: builder.minecraft_version().to_string(),
                loader_version: builder.loader_version().to_string(),
            },
        ));
    }

    lighty_core::trace_info!(
        "[Launch] Metadata fetched successfully for {:?}",
        builder.loader()
    );
    Ok(metadata)
}

/// Ensures Java is installed for `version` and returns the binary path.
async fn ensure_java_installed<T>(
    builder: &T,
    version: &Version,
    java_distribution: &JavaDistribution,
    #[cfg(feature = "events")] event_bus: Option<&EventBus>,
) -> InstallerResult<PathBuf>
where
    T: VersionInfo,
{
    let java_version = version.java_version.major_version;

    // Look for an existing Java install before downloading
    match find_java_binary(builder.java_dirs(), java_distribution, &java_version).await {
        Ok(path) => {
            lighty_core::trace_info!(
                "[Java] Java {} already installed at: {:?}",
                java_version,
                path
            );

            #[cfg(feature = "events")]
            if let Some(bus) = event_bus {
                bus.emit(lighty_event::Event::Java(
                    lighty_event::JavaEvent::JavaAlreadyInstalled {
                        distribution: java_distribution.get_name().to_string(),
                        version: java_version,
                        binary_path: path.to_string_lossy().to_string(),
                    },
                ));
            }

            Ok(path)
        }
        Err(_) => {
            lighty_core::trace_info!("[Java] Java {} not found, downloading...", java_version);

            #[cfg(feature = "events")]
            if let Some(bus) = event_bus {
                bus.emit(lighty_event::Event::Java(
                    lighty_event::JavaEvent::JavaNotFound {
                        distribution: java_distribution.get_name().to_string(),
                        version: java_version,
                    },
                ));
            }

            #[cfg(feature = "events")]
            let path = jre_download(
                builder.java_dirs(),
                java_distribution,
                &java_version,
                |current, total| {
                    lighty_core::trace_debug!("[Java] Download progress: {}/{}", current, total);
                },
                event_bus,
            )
            .await
            .map_err(|e| InstallerError::DownloadFailed(format!("JRE download failed: {}", e)))?;

            #[cfg(not(feature = "events"))]
            let path = jre_download(
                builder.java_dirs(),
                java_distribution,
                &java_version,
                |current, total| {
                    lighty_core::trace_debug!("[Java] Download progress: {}/{}", current, total);
                },
            )
            .await
            .map_err(|e: JreError| {
                InstallerError::DownloadFailed(format!("JRE download failed: {}", e))
            })?;

            lighty_core::trace_info!("[Java] Java {} installed successfully", java_version);
            Ok(path)
        }
    }
}

/// Spawns the game process and wires up event/console handlers.
async fn execute_game<T>(
    builder: &T,
    version: &Version,
    profile: &UserProfile,
    java_path: PathBuf,
    arg_overrides: &HashMap<String, String>,
    arg_removals: &HashSet<String>,
    jvm_overrides: &HashMap<String, String>,
    jvm_removals: &HashSet<String>,
    raw_args: &[String],
    #[cfg(feature = "events")] event_bus: Option<&EventBus>,
) -> InstallerResult<()>
where
    T: VersionInfo + Arguments,
{
    use crate::instance::manager::GameInstance;
    use crate::instance::{handle_console_streams, INSTANCE_MANAGER};

    let username = profile.username.as_str();

    // Build the full argv (JVM args + main class + game args)
    let arguments = builder.build_arguments(
        version,
        Some(profile),
        arg_overrides,
        arg_removals,
        jvm_overrides,
        jvm_removals,
        raw_args,
    );

    // Wrap the Java binary path in a runtime helper
    let java_runtime = JavaRuntime::new(java_path);
    lighty_core::trace_info!("[Launch] Executing game...");

    match java_runtime.execute(arguments, builder.game_dirs()).await {
        Ok(child) => {
            let pid = child.id().ok_or(InstallerError::NoPid)?;

            lighty_core::trace_info!("[Launch] Game launched successfully, PID: {}", pid);

            // Register the instance (metadata only — the child is owned by the console task)
            let instance = GameInstance {
                pid,
                instance_name: builder.name().to_string(),
                version: format!(
                    "{}-{}",
                    builder.minecraft_version(),
                    builder.loader_version()
                ),
                username: username.to_string(),
                game_dir: builder.game_dirs().to_path_buf(),
                started_at: std::time::SystemTime::now(),
            };

            if let Err(e) = INSTANCE_MANAGER.register_instance(instance).await {
                lighty_core::trace_warn!(
                    error = %e,
                    "Failed to register launched instance — process keeps running"
                );
            }

            // Emit InstanceLaunched event
            #[cfg(feature = "events")]
            if let Some(bus) = event_bus {
                use lighty_event::{Event, InstanceLaunchedEvent};

                bus.emit(Event::InstanceLaunched(InstanceLaunchedEvent {
                    pid,
                    instance_name: builder.name().to_string(),
                    version: format!(
                        "{}-{}",
                        builder.minecraft_version(),
                        builder.loader_version()
                    ),
                    username: username.to_string(),
                    timestamp: std::time::SystemTime::now(),
                }));

                // Spawn the window-appearance watcher
                let bus_clone = bus.clone();
                let instance_name = builder.name().to_string();
                let version = format!(
                    "{}-{}",
                    builder.minecraft_version(),
                    builder.loader_version()
                );
                tokio::spawn(super::window::detect_window_appearance(
                    pid,
                    instance_name,
                    version,
                    bus_clone,
                ));
            }

            // Spawn the console-streaming handler. It takes ownership of the
            // child and handles all stdio until the process exits.
            tokio::spawn(handle_console_streams(
                pid,
                builder.name().to_string(),
                child,
                #[cfg(feature = "events")]
                event_bus.cloned(),
            ));

            Ok(())
        }
        Err(e) => {
            lighty_core::trace_error!("[Launch] Failed to launch game: {}", e);
            Err(InstallerError::DownloadFailed(format!(
                "Launch failed: {}",
                e
            )))
        }
    }
}

/// Extracts the [`Version`] payload from a [`VersionMetaData`] variant.
fn extract_version(metadata: &VersionMetaData) -> InstallerResult<&Version> {
    match metadata {
        VersionMetaData::Version(v) => Ok(v),
        _ => Err(InstallerError::InvalidMetadata),
    }
}
