// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! LightyLauncher - A modern Minecraft launcher library.
//!
//! ```no_run
//! use lighty_launcher::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     AppState::init("MyLauncher")?;
//!     let mut auth = auth::OfflineAuth::new("Player");
//!     let profile = auth.authenticate().await?;
//!     let mut version = version::VersionBuilder::new(
//!         "my-instance", Loader::Vanilla, "", "1.21.1",
//!     );
//!     version.launch(&profile, JavaDistribution::Temurin).run().await?;
//!     Ok(())
//! }
//! ```

pub mod auth {
    //! Authentication providers and utilities.

    pub use lighty_auth::{
        Authenticator,
        UserProfile,
        UserRole,
        AuthProvider,
        AuthResult,
        AuthError,
        generate_offline_uuid,
        offline::OfflineAuth,
        microsoft::MicrosoftAuth,
        azuriom::AzuriomAuth,
    };
}

#[cfg(feature = "events")]
pub mod event {
    //! Event system for tracking launcher operations.

    pub use lighty_event::{
        EventBus,
        EventReceiver,
        Event,
        AuthEvent,
        JavaEvent,
        LaunchEvent,
        LoaderEvent,
        CoreEvent,
        InstanceLaunchedEvent,
        InstanceExitedEvent,
        ConsoleOutputEvent,
        InstanceDeletedEvent,
        ConsoleStream,
        EventReceiveError,
        EventTryReceiveError,
        EventSendError,
        EVENT_BUS,
    };
}

pub mod java {
    //! Java runtime management.

    pub use lighty_java::{
        JavaDistribution,
        DistributionSelection,
        runtime::JavaRuntime,
        jre_downloader,
        JreError,
        JreResult,
        JavaRuntimeError,
        JavaRuntimeResult,
        DistributionError,
        DistributionResult,
    };
}

pub mod launch {
    //! Game launching and installation.

    pub use lighty_launch::{
        launch::{Launch, LaunchBuilder, LaunchConfig},
        installer::{
            Installer,
            config::{DownloaderConfig, init_downloader_config},
        },
        arguments::Arguments as LaunchArguments,
        errors::{InstallerError, InstallerResult},
        InstanceControl,
        InstanceError,
        InstanceResult,
    };

    /// Launch argument keys for customization
    pub mod keys {
        pub use lighty_launch::arguments::{
            KEY_AUTH_PLAYER_NAME,
            KEY_AUTH_UUID,
            KEY_AUTH_ACCESS_TOKEN,
            KEY_AUTH_XUID,
            KEY_CLIENT_ID,
            KEY_USER_TYPE,
            KEY_USER_PROPERTIES,
            KEY_VERSION_NAME,
            KEY_VERSION_TYPE,
            KEY_GAME_DIRECTORY,
            KEY_ASSETS_ROOT,
            KEY_NATIVES_DIRECTORY,
            KEY_LIBRARY_DIRECTORY,
            KEY_ASSETS_INDEX_NAME,
            KEY_LAUNCHER_NAME,
            KEY_LAUNCHER_VERSION,
            KEY_CLASSPATH,
            KEY_CLASSPATH_SEPARATOR,
        };
    }
}

pub mod loaders {
    //! Minecraft mod loaders and version metadata.

    pub use lighty_loaders::{
        types::{
            Loader,
            VersionInfo,
            LoaderExtensions,
            InstanceSize,
            version_metadata::{
                Version,
                VersionMetaData,
                Library,
                Asset,
                AssetIndex,
                Arguments,
                MainClass,
                Mods,
                Native,
            },
        },
        utils::{cache, manifest, query},
    };

    #[cfg(feature = "vanilla")]
    pub use lighty_loaders::loaders::vanilla;
    #[cfg(feature = "fabric")]
    pub use lighty_loaders::loaders::fabric;
    #[cfg(feature = "quilt")]
    pub use lighty_loaders::loaders::quilt;
    #[cfg(feature = "forge")]
    pub use lighty_loaders::loaders::forge;
    #[cfg(feature = "neoforge")]
    pub use lighty_loaders::loaders::neoforge;
    #[cfg(feature = "lighty_updater")]
    pub use lighty_loaders::loaders::lighty_updater;
    pub use lighty_loaders::loaders::optifine;

    // Back-compat alias: `lighty_launcher::loaders::mods` continues to
    // resolve to the (now standalone) modsloader crate. New code should
    // prefer `lighty_launcher::mods` (top-level, see crate root).
    #[cfg(any(feature = "modrinth", feature = "curseforge"))]
    pub use lighty_modsloader as mods;
}

/// Mods + modpacks (Modrinth / CurseForge / `.mrpack` / CurseForge `.zip`).
#[cfg(any(feature = "modrinth", feature = "curseforge"))]
pub use lighty_modsloader as mods;

pub mod version {
    //! Version builders for game instances.

    pub use lighty_version::{
        VersionBuilder,
        LightyVersionBuilder,
    };
}

pub mod core {
    //! Core utilities and system operations.

    pub use lighty_core::{
        system,
        hosts,
        download,
        extract,
        hash,
        app_state::AppState,
        errors::{AppStateError, AppStateResult},
        SystemError,
        SystemResult,
        ExtractError,
        ExtractResult,
        DownloadError,
        DownloadResult,
        HashError,
        HashResult,
        verify_file_sha1,
        verify_file_sha1_streaming,
        calculate_file_sha1_sync,
        verify_file_sha1_sync,
        calculate_sha1_bytes,
        calculate_sha1_bytes_raw,
    };
}

pub mod macros {
    //! Utility macros — conditional tracing + filesystem helpers.

    pub use lighty_core::{
        trace_debug,
        trace_info,
        trace_warn,
        trace_error,
        time_it,
        mkdir,
        join_and_mkdir,
        join_and_mkdir_vec,
        mkdir_blocking,
    };
}

pub mod prelude {
    //! Convenient re-exports of most commonly used types.

    pub use crate::auth::{
        Authenticator, UserProfile, AuthProvider, AuthError, UserRole,
        OfflineAuth, MicrosoftAuth, AzuriomAuth,
    };

    #[cfg(feature = "events")]
    pub use crate::event::{
        EventBus, Event, AuthEvent, JavaEvent, LaunchEvent, LoaderEvent, CoreEvent,
        InstanceLaunchedEvent, InstanceExitedEvent, ConsoleOutputEvent, InstanceDeletedEvent,
        ConsoleStream, EVENT_BUS,
    };

    pub use crate::java::JavaDistribution;

    pub use crate::launch::{
        Launch, LaunchBuilder, DownloaderConfig, init_downloader_config,
        InstanceControl, InstanceError, InstanceResult,
        LaunchArguments,
    };
    pub use crate::launch::keys::*;

    pub use crate::loaders::{Loader, VersionInfo, LoaderExtensions, InstanceSize};

    // `InstanceCache` brings `instance.clear_cache().await` into scope.
    pub use lighty_modsloader::{InstanceCache, ModRequest, WithMods};
    #[cfg(any(feature = "modrinth", feature = "curseforge"))]
    pub use lighty_modsloader::ModpackSource;

    pub use crate::version::{VersionBuilder, LightyVersionBuilder};

    pub use crate::core::AppState;

    pub use crate::macros::{trace_debug, trace_info, trace_warn, trace_error};
}

pub use loaders::Loader;
pub use java::JavaDistribution;
pub use launch::Launch;
pub use auth::{Authenticator, UserProfile};
pub use version::{VersionBuilder, LightyVersionBuilder};

#[doc(hidden)]
pub use lighty_core as _core;
#[doc(hidden)]
pub use lighty_auth as _auth;
#[cfg(feature = "events")]
#[doc(hidden)]
pub use lighty_event as _event;
#[doc(hidden)]
pub use lighty_java as _java;
#[doc(hidden)]
pub use lighty_launch as _launch;
#[doc(hidden)]
pub use lighty_loaders as _loaders;
#[doc(hidden)]
pub use lighty_version as _version;
