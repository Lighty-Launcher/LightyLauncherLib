use std::path::{Path, PathBuf};

use lighty_core::AppState;
use lighty_loaders::types::{Loader, VersionInfo};
use lighty_modsloader::{ModRequest, WithMods};

#[cfg(any(feature = "modrinth", feature = "curseforge"))]
use lighty_modsloader::ModpackSource;

/// Builder for LightyUpdater-managed instances.
///
/// Unlike [`super::VersionBuilder`], `loader_version` here holds the
/// LightyUpdater server URL: the actual loader and Minecraft version
/// are fetched from that server at install time. Default paths come
/// from the global [`AppState`] (call [`AppState::init`] first).
///
/// User-attached Modrinth / CurseForge mods can be layered on top of
/// the server-pushed metadata via [`with_mod`]. Modpacks are
/// intentionally not supported here — LightyUpdater is itself the
/// modpack source of truth, mixing in a second modpack would conflict
/// on the loader / Minecraft version.
///
/// [`with_mod`]: LightyVersionBuilder::with_mod
#[derive(Debug, Clone)]
pub struct LightyVersionBuilder {
    pub name: String,
    pub server_url: String,
    pub minecraft_version: Option<String>,
    pub loader: Option<Loader>,
    pub game_dirs: PathBuf,
    pub java_dirs: PathBuf,
    pub mod_requests: Vec<ModRequest>,
}

impl LightyVersionBuilder {
    /// Creates a new `LightyVersionBuilder`.
    ///
    /// `server_url` is the LightyUpdater server endpoint; the loader
    /// and Minecraft version are resolved from its response at install
    /// time. Panics if [`AppState::init`] hasn't been called.
    ///
    /// # Example
    /// ```rust,no_run
    /// use lighty_core::AppState;
    /// use lighty_version::LightyVersionBuilder;
    ///
    /// AppState::init("MyLauncher").unwrap();
    /// let builder = LightyVersionBuilder::new("my-server", "https://updater.example.com");
    /// ```
    pub fn new(name: &str, server_url: &str) -> Self {
        Self {
            name: name.to_string(),
            server_url: server_url.to_string(),
            minecraft_version: None,
            loader: None,
            game_dirs: AppState::data_dir().join(name),
            java_dirs: AppState::config_dir().join("jre"),
            mod_requests: Vec::new(),
        }
    }

    /// Opens the user-mods sub-builder. Modpacks are not exposed on
    /// purpose — see the struct doc.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use lighty_core::AppState;
    /// # use lighty_version::LightyVersionBuilder;
    /// # AppState::init("MyLauncher").unwrap();
    /// let builder = LightyVersionBuilder::new("server", "https://updater.example.com")
    ///     .with_mod()
    ///         .done();
    /// ```
    pub fn with_mod(self) -> LightyModSourcesBuilder {
        LightyModSourcesBuilder {
            parent: self,
            pending: Vec::new(),
        }
    }
}

impl VersionInfo for LightyVersionBuilder {
    type LoaderType = Loader;

    fn name(&self) -> &str {
        &self.name
    }

    fn loader_version(&self) -> &str {
        &self.server_url
    }

    fn minecraft_version(&self) -> &str {
        self.minecraft_version.as_ref().map_or("", String::as_str)
    }

    fn game_dirs(&self) -> &Path {
        &self.game_dirs
    }

    fn java_dirs(&self) -> &Path {
        &self.java_dirs
    }

    fn loader(&self) -> &Self::LoaderType {
        self.loader.as_ref().unwrap_or(&Loader::LightyUpdater)
    }
}

impl<'b> VersionInfo for &'b LightyVersionBuilder {
    type LoaderType = Loader;

    fn name(&self) -> &str {
        &self.name
    }

    fn loader_version(&self) -> &str {
        &self.server_url
    }

    fn minecraft_version(&self) -> &str {
        self.minecraft_version.as_ref().map_or("", String::as_str)
    }

    fn game_dirs(&self) -> &Path {
        &self.game_dirs
    }

    fn java_dirs(&self) -> &Path {
        &self.java_dirs
    }

    fn loader(&self) -> &Self::LoaderType {
        self.loader.as_ref().unwrap_or(&Loader::LightyUpdater)
    }
}

impl WithMods for LightyVersionBuilder {
    fn mod_requests(&self) -> &[ModRequest] {
        &self.mod_requests
    }

    #[cfg(any(feature = "modrinth", feature = "curseforge"))]
    fn modpack(&self) -> Option<&ModpackSource> {
        None
    }
}

impl<'b> WithMods for &'b LightyVersionBuilder {
    fn mod_requests(&self) -> &[ModRequest] {
        &self.mod_requests
    }

    #[cfg(any(feature = "modrinth", feature = "curseforge"))]
    fn modpack(&self) -> Option<&ModpackSource> {
        None
    }
}

/// Sub-builder accumulating user-attached Modrinth / CurseForge mods
/// on a [`LightyVersionBuilder`]. No modpack methods — LightyUpdater
/// is already the modpack source.
pub struct LightyModSourcesBuilder {
    parent: LightyVersionBuilder,
    pending: Vec<ModRequest>,
}

impl LightyModSourcesBuilder {
    /// Adds Modrinth mod requests on top of the LightyUpdater-pushed metadata.
    ///
    /// Each tuple is `(project-slug-or-id, optional-mod-version-id)`.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use lighty_core::AppState;
    /// # use lighty_version::LightyVersionBuilder;
    /// # AppState::init("MyLauncher").unwrap();
    /// let builder = LightyVersionBuilder::new("server", "https://updater.example.com")
    ///     .with_mod()
    ///         .with_modrinth_mods(vec![
    ///             ("sodium-extra", None),
    ///             ("iris", None),
    ///         ])
    ///         .done();
    /// ```
    #[cfg(feature = "modrinth")]
    pub fn with_modrinth_mods<S>(mut self, list: Vec<(S, Option<String>)>) -> Self
    where
        S: Into<String>,
    {
        for (id_or_slug, version) in list {
            self.pending.push(ModRequest::Modrinth {
                id_or_slug: id_or_slug.into(),
                version,
            });
        }
        self
    }

    /// Adds CurseForge mod requests on top of the LightyUpdater-pushed metadata.
    ///
    /// Requires [`lighty_modsloader::curseforge::set_api_key`] to have
    /// been called before `.run()`.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use lighty_core::AppState;
    /// # use lighty_version::LightyVersionBuilder;
    /// # AppState::init("MyLauncher").unwrap();
    /// let builder = LightyVersionBuilder::new("server", "https://updater.example.com")
    ///     .with_mod()
    ///         .with_curseforge_mods(vec![(238222, None)])
    ///         .done();
    /// ```
    #[cfg(feature = "curseforge")]
    pub fn with_curseforge_mods(mut self, list: Vec<(u32, Option<u32>)>) -> Self {
        for (mod_id, file_id) in list {
            self.pending.push(ModRequest::CurseForge { mod_id, file_id });
        }
        self
    }

    /// Threads the accumulated mod requests back into the parent builder.
    pub fn done(self) -> LightyVersionBuilder {
        let mut parent = self.parent;
        let mut pending = self.pending;
        parent.mod_requests.append(&mut pending);
        parent
    }
}
