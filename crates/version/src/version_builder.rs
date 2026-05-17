use std::{
    fmt::Debug,
    path::{Path, PathBuf},
    time::Duration,
};

use lighty_core::AppState;
use lighty_loaders::types::VersionInfo;
use lighty_modsloader::{ModRequest, WithMods};

#[cfg(any(feature = "modrinth", feature = "curseforge"))]
use lighty_modsloader::ModpackSource;

/// Configures a Minecraft instance: name, loader, versions, and on-disk paths.
///
/// Default directories are derived from the global [`AppState`]:
/// - `game_dirs`   = `AppState::data_dir().join(name)`
/// - `runtime_dir` = alias of `game_dirs` until overridden
/// - `java_dirs`   = `AppState::config_dir().join("jre")`
///
/// Call [`AppState::init`] once at startup before constructing any
/// `VersionBuilder`.
#[derive(Debug, Clone)]
pub struct VersionBuilder<L = ()> {
    pub name: String,
    pub loader: L,
    pub loader_version: String,
    pub minecraft_version: String,
    pub game_dirs: PathBuf,
    pub java_dirs: PathBuf,
    pub runtime_dir: PathBuf,
    pub mod_requests: Vec<ModRequest>,
    #[cfg(any(feature = "modrinth", feature = "curseforge"))]
    pub modpack: Option<ModpackSource>,
    pub ttl_override: Option<Duration>,
}

impl<L> VersionBuilder<L> {
    /// Creates a new `VersionBuilder` with default paths derived
    /// from the global [`AppState`].
    ///
    /// Panics if [`AppState::init`] hasn't been called yet.
    ///
    /// # Example
    /// ```rust,no_run
    /// use lighty_core::AppState;
    /// use lighty_loaders::types::Loader;
    /// use lighty_version::VersionBuilder;
    ///
    /// AppState::init("MyLauncher").unwrap();
    /// let builder = VersionBuilder::new("my-instance", Loader::Vanilla, "", "1.21.1");
    /// ```
    pub fn new(
        name: &str,
        loader: L,
        loader_version: &str,
        minecraft_version: &str,
    ) -> Self {
        let game_dirs = AppState::data_dir().join(name);
        let java_dirs = AppState::config_dir().join("jre");
        Self {
            name: name.to_string(),
            loader,
            loader_version: loader_version.to_string(),
            minecraft_version: minecraft_version.to_string(),
            runtime_dir: game_dirs.clone(),
            game_dirs,
            java_dirs,
            mod_requests: Vec::new(),
            #[cfg(any(feature = "modrinth", feature = "curseforge"))]
            modpack: None,
            ttl_override: None,
        }
    }

    /// Opens the mod-sources sub-builder.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use lighty_core::AppState;
    /// # use lighty_loaders::types::Loader;
    /// # use lighty_version::VersionBuilder;
    /// # AppState::init("MyLauncher").unwrap();
    /// let builder = VersionBuilder::new("modded", Loader::Fabric, "0.16.0", "1.21.1")
    ///     .with_mod()
    ///         .done();
    /// ```
    pub fn with_mod(self) -> ModSourcesBuilder<L> {
        ModSourcesBuilder {
            parent: self,
            pending: Vec::new(),
            #[cfg(any(feature = "modrinth", feature = "curseforge"))]
            pending_modpack: None,
        }
    }

    /// Overrides the Java install directory.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use lighty_core::AppState;
    /// # use lighty_loaders::types::Loader;
    /// # use lighty_version::VersionBuilder;
    /// use std::path::PathBuf;
    /// # AppState::init("MyLauncher").unwrap();
    /// let builder = VersionBuilder::new("my-instance", Loader::Vanilla, "", "1.21.1")
    ///     .with_custom_java_dir(PathBuf::from("/opt/java"));
    /// ```
    pub fn with_custom_java_dir(mut self, java_dir: PathBuf) -> Self {
        self.java_dirs = java_dir;
        self
    }

    /// Replaces the loader.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use lighty_core::AppState;
    /// # use lighty_loaders::types::Loader;
    /// # use lighty_version::VersionBuilder;
    /// # AppState::init("MyLauncher").unwrap();
    /// let builder = VersionBuilder::new("my-instance", Loader::Vanilla, "", "1.21.1")
    ///     .with_loader(Loader::Fabric);
    /// ```
    pub fn with_loader(mut self, loader: L) -> Self {
        self.loader = loader;
        self
    }

    /// Replaces the loader version.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use lighty_core::AppState;
    /// # use lighty_loaders::types::Loader;
    /// # use lighty_version::VersionBuilder;
    /// # AppState::init("MyLauncher").unwrap();
    /// let builder = VersionBuilder::new("my-instance", Loader::Fabric, "0.16.0", "1.21.1")
    ///     .with_loader_version("0.17.2");
    /// ```
    pub fn with_loader_version(mut self, version: &str) -> Self {
        self.loader_version = version.to_string();
        self
    }

    /// Replaces the Minecraft version.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use lighty_core::AppState;
    /// # use lighty_loaders::types::Loader;
    /// # use lighty_version::VersionBuilder;
    /// # AppState::init("MyLauncher").unwrap();
    /// let builder = VersionBuilder::new("my-instance", Loader::Vanilla, "", "1.21.1")
    ///     .with_minecraft_version("1.20.4");
    /// ```
    pub fn with_minecraft_version(mut self, version: &str) -> Self {
        self.minecraft_version = version.to_string();
        self
    }

    /// Overrides the TTL applied to every cache associated with this
    /// instance. Default = 24h.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use lighty_core::AppState;
    /// # use lighty_loaders::types::Loader;
    /// # use lighty_version::VersionBuilder;
    /// use std::time::Duration;
    /// # AppState::init("MyLauncher").unwrap();
    /// let builder = VersionBuilder::new("my-instance", Loader::Vanilla, "", "1.21.1")
    ///     .with_ttl_duration(Duration::from_secs(3600));
    /// ```
    pub fn with_ttl_duration(mut self, ttl: Duration) -> Self {
        self.ttl_override = Some(ttl);
        self
    }
}

impl<L: Clone + Send + Sync + Debug> VersionInfo for VersionBuilder<L> {
    type LoaderType = L;

    fn name(&self) -> &str {
        &self.name
    }

    fn loader_version(&self) -> &str {
        &self.loader_version
    }

    fn minecraft_version(&self) -> &str {
        &self.minecraft_version
    }

    fn game_dirs(&self) -> &Path {
        &self.game_dirs
    }

    fn java_dirs(&self) -> &Path {
        &self.java_dirs
    }

    fn loader(&self) -> &Self::LoaderType {
        &self.loader
    }

    fn runtime_dir(&self) -> &Path {
        &self.runtime_dir
    }

    fn set_runtime_dir(&mut self, path: PathBuf) {
        self.runtime_dir = path;
    }

    fn ttl(&self) -> Duration {
        self.ttl_override.unwrap_or(Duration::from_secs(86_400))
    }
}

// Read-only impl on &VersionBuilder: the no-op `set_runtime_dir` default
// applies because we can't mutate through a shared reference.
impl<'b, L: Clone + Send + Sync + Debug> VersionInfo for &'b VersionBuilder<L> {
    type LoaderType = L;

    fn name(&self) -> &str {
        &self.name
    }

    fn loader_version(&self) -> &str {
        &self.loader_version
    }

    fn minecraft_version(&self) -> &str {
        &self.minecraft_version
    }

    fn game_dirs(&self) -> &Path {
        &self.game_dirs
    }

    fn java_dirs(&self) -> &Path {
        &self.java_dirs
    }

    fn loader(&self) -> &Self::LoaderType {
        &self.loader
    }

    fn runtime_dir(&self) -> &Path {
        &self.runtime_dir
    }

    fn ttl(&self) -> Duration {
        self.ttl_override.unwrap_or(Duration::from_secs(86_400))
    }
}

impl<L: Clone + Send + Sync + Debug> WithMods for VersionBuilder<L> {
    fn mod_requests(&self) -> &[ModRequest] {
        &self.mod_requests
    }

    #[cfg(any(feature = "modrinth", feature = "curseforge"))]
    fn modpack(&self) -> Option<&ModpackSource> {
        self.modpack.as_ref()
    }
}

impl<'b, L: Clone + Send + Sync + Debug> WithMods for &'b VersionBuilder<L> {
    fn mod_requests(&self) -> &[ModRequest] {
        &self.mod_requests
    }

    #[cfg(any(feature = "modrinth", feature = "curseforge"))]
    fn modpack(&self) -> Option<&ModpackSource> {
        self.modpack.as_ref()
    }
}

/// Sub-builder accumulating mod sources and an optional modpack.
pub struct ModSourcesBuilder<L> {
    parent: VersionBuilder<L>,
    pending: Vec<ModRequest>,
    #[cfg(any(feature = "modrinth", feature = "curseforge"))]
    pending_modpack: Option<ModpackSource>,
}

impl<L> ModSourcesBuilder<L> {
    /// Adds Modrinth mod requests.
    ///
    /// Each tuple is `(project-slug-or-id, optional-mod-version-id)`.
    /// `version` is the Modrinth release id, not the Minecraft version.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use lighty_core::AppState;
    /// # use lighty_loaders::types::Loader;
    /// # use lighty_version::VersionBuilder;
    /// # AppState::init("MyLauncher").unwrap();
    /// let builder = VersionBuilder::new("modded", Loader::Fabric, "0.17.2", "1.21.1")
    ///     .with_mod()
    ///         .with_modrinth_mods(vec![
    ///             ("sodium", None),
    ///             ("lithium", Some("mc1.21.1-0.13.0".into())),
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

    /// Adds CurseForge mod requests.
    ///
    /// Each tuple is `(numeric-mod-id, optional-numeric-file-id)`.
    /// Requires [`lighty_modsloader::curseforge::set_api_key`] to
    /// have been called before `.run()`.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use lighty_core::AppState;
    /// # use lighty_loaders::types::Loader;
    /// # use lighty_version::VersionBuilder;
    /// # AppState::init("MyLauncher").unwrap();
    /// let builder = VersionBuilder::new("modded", Loader::Fabric, "0.17.2", "1.21.1")
    ///     .with_mod()
    ///         .with_curseforge_mods(vec![
    ///             (238222, None),       // JEI, latest compatible
    ///             (306612, Some(4000000)),
    ///         ])
    ///         .done();
    /// ```
    #[cfg(feature = "curseforge")]
    pub fn with_curseforge_mods(mut self, list: Vec<(u32, Option<u32>)>) -> Self {
        for (mod_id, file_id) in list {
            self.pending.push(ModRequest::CurseForge { mod_id, file_id });
        }
        self
    }

    /// Attaches a Modrinth `.mrpack` modpack.
    ///
    /// Accepts either a CDN URL or an explicit
    /// [`ModpackSource::ModrinthPinned`] for `(project, version_id)` pinning.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use lighty_core::AppState;
    /// # use lighty_loaders::types::Loader;
    /// # use lighty_version::VersionBuilder;
    /// use lighty_modsloader::ModpackSource;
    /// # AppState::init("MyLauncher").unwrap();
    /// let builder = VersionBuilder::new("pack", Loader::Fabric, "0.17.2", "1.21.1")
    ///     .with_mod()
    ///         .with_modrinth_modpack(ModpackSource::ModrinthPinned {
    ///             project: "fabulously-optimized".into(),
    ///             version: Some("5.10.0".into()),
    ///         })
    ///         .done();
    /// ```
    #[cfg(feature = "modrinth")]
    pub fn with_modrinth_modpack(mut self, source: impl Into<ModpackSource>) -> Self {
        self.pending_modpack = Some(source.into());
        self
    }

    /// Attaches a CurseForge `.zip` modpack by `(project_id, file_id)`.
    ///
    /// Requires [`lighty_modsloader::curseforge::set_api_key`] to have
    /// been called before `.run()`.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use lighty_core::AppState;
    /// # use lighty_loaders::types::Loader;
    /// # use lighty_version::VersionBuilder;
    /// # AppState::init("MyLauncher").unwrap();
    /// let builder = VersionBuilder::new("pack", Loader::Forge, "47.3.0", "1.20.1")
    ///     .with_mod()
    ///         .with_curseforge_modpack(715572, 4769518)
    ///         .done();
    /// ```
    #[cfg(feature = "curseforge")]
    pub fn with_curseforge_modpack(mut self, project_id: u32, file_id: u32) -> Self {
        self.pending_modpack = Some(ModpackSource::CurseForgePinned { project_id, file_id });
        self
    }

    /// Threads the accumulated mod requests and modpack source back
    /// into the parent builder.
    pub fn done(self) -> VersionBuilder<L> {
        let mut parent = self.parent;
        let mut pending = self.pending;
        parent.mod_requests.append(&mut pending);
        #[cfg(any(feature = "modrinth", feature = "curseforge"))]
        if self.pending_modpack.is_some() {
            parent.modpack = self.pending_modpack;
        }
        parent
    }
}
