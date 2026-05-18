# Launch

The launch pipeline ties everything together: fetch loader metadata,
ensure Java is present, install the 8 buckets in parallel, build the
argv, spawn the JVM, register the PID, stream stdio.

```text
.launch(profile, java).run()
    │
    ▼
1. Prepare metadata          (loader-specific HTTP fetch + parse)
2. Ensure Java installed     (download if missing; lighty-java)
3. Install dependencies      (8 parallel buckets; see installation.md)
4. Forge/NeoForge post-hook  (install_profile libs + processors)
5. Build argv                (placeholders + JVM + game args)
6. Spawn JVM                 (Command::spawn; register PID)
7. Stream stdio + lifecycle  (see instance-lifecycle.md)
```

## The `Launch` trait

Adds `.launch(...)` to any installable instance:

```rust,ignore
pub trait Launch {
    fn launch<'a>(
        &'a mut self,
        profile: &'a UserProfile,
        java_distribution: JavaDistribution,
    ) -> LaunchBuilder<'a, Self>
    where Self: Sized;
}
```

Blanket-implemented for every type that satisfies the pipeline's
bounds (`VersionInfo<LoaderType = Loader> + LoaderExtensions +
Arguments + Installer + WithMods`) — that's `VersionBuilder` and
`LightyVersionBuilder` out of the box. `WithMods` returns an empty
slice on vanilla instances, so it's free.

## `LaunchBuilder` API

```rust,ignore
pub struct LaunchBuilder<'a, T> { /* … */ }

impl<'a, T> LaunchBuilder<'a, T>
where T: VersionInfo<LoaderType = Loader> + LoaderExtensions + Arguments + Installer + WithMods
{
    pub fn with_jvm_options(self) -> JvmOptionsBuilder<'a, T>;
    pub fn with_arguments  (self) -> ArgumentsBuilder<'a, T>;

    #[cfg(feature = "events")]
    pub fn with_event_bus(self, bus: &'a EventBus) -> Self;

    pub async fn run(self) -> InstallerResult<()>;
}
```

`with_jvm_options()` / `with_arguments()` open sub-builders that take
`.set(key, value)` / `.remove(key)` / `.done()` to return to the
parent. See [arguments.md](./arguments.md) for the placeholder catalogue.

## Step-by-step

### 1. Prepare metadata

Dispatches to the loader's `get_metadata()` (Vanilla, Fabric, Quilt,
Forge, NeoForge, …). With `events` enabled, emits
`LoaderEvent::FetchingData` then `LoaderEvent::DataFetched`. Per-loader
detail and the actual HTTP endpoints live in
[`crates/loaders/docs/loaders/`](../../loaders/docs/loaders/).

### 2. Ensure Java installed

Extracts `version.java_version.major_version` from the metadata, asks
[`lighty-java`](../../java/docs/overview.md) for a matching JRE. If
missing, downloads from the requested distribution (Temurin, Zulu,
Graal, Liberica) and extracts to `java_dirs()/jre/java-<v>/`. Emits
`JavaEvent::{JavaAlreadyInstalled, JavaNotFound, …}`.

### 3. Install dependencies

8-way parallel `tokio::try_join!` across libraries, natives, client
JAR, assets, mods, resource packs, shader packs, datapacks. Optional
modpack pre-step runs before user-mod resolution. Full pipeline +
SHA1 verification + per-bucket layout in
[installation.md](./installation.md).

### 4. Forge / NeoForge post-hook

For `Loader::Forge` (modern, 1.13+) and `Loader::NeoForge`: download
the `install_profile.json` libraries through the shared library
installer, then run the install processors. Legacy Forge (1.7.10 –
1.12.2) skips processors — the universal JAR ships inside the
installer and is extracted to its Maven path. Per-loader detail in
[`crates/loaders/docs/loaders/forge.md`](../../loaders/docs/loaders/forge.md)
and [`neoforge.md`](../../loaders/docs/loaders/neoforge.md).

> The `forge` Cargo feature covers **both** modern and legacy Forge in
> a single switch — there's no separate `forge_legacy` feature.

### 5. Build argv

Variable map → JVM args (defaults if metadata is empty) → critical
JVM injections (`-Djava.library.path=...`, launcher brand / version,
`-cp ...`) → game args. Detailed in [arguments.md](./arguments.md);
the access-token routing (in particular how the `keyring` feature
keeps the secret out of process memory) is documented there too.

### 6. Spawn JVM

```rust,ignore
Command::new(java_path)
    .args(arguments)
    .current_dir(builder.game_dirs())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()?;
```

Emits `LaunchEvent::Launched { version, pid }` on success (or
`NotLaunched { error }` on failure). `InstallerError::NoPid` is
returned if `child.id()` is `None`.

### 7. Register + stream

The new `GameInstance` is registered in the global
[`InstanceManager`](./instance-lifecycle.md#instance-manager) keyed by
PID. A dedicated task takes ownership of the `Child` and:

- streams stdout/stderr line-by-line as
  `LaunchEvent::ProcessOutput { pid, stream, line }`,
- waits on the exit code,
- emits `LaunchEvent::ProcessExited { pid, exit_code }`,
- unregisters the instance from the manager.

Full state-machine + manager internals: [instance-lifecycle.md](./instance-lifecycle.md).

## Platform differences

| Platform | Java exec | Classpath sep | Process kill | Natives |
|---|---|---|---|---|
| Windows | `java.exe` | `;` | `taskkill /PID {pid} /F` | `…-natives-windows.jar` |
| Linux | `java` | `:` | `kill -SIGTERM {pid}` | `…-natives-linux.jar` |
| macOS | `java` | `:` | `kill -SIGTERM {pid}` | `…-natives-macos.jar` |

`-XstartOnFirstThread` is auto-injected on macOS (LWJGL / GLFW
requirement).

## Complete example

```rust,no_run
use lighty_auth::{offline::OfflineAuth, Authenticator};
use lighty_core::AppState;
use lighty_java::JavaDistribution;
use lighty_launch::launch::Launch;
use lighty_launch::InstanceControl;
use lighty_loaders::types::Loader;
use lighty_version::VersionBuilder;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("MyLauncher")?;

    let mut instance = VersionBuilder::new(
        "fabric-1.21",
        Loader::Fabric,
        "0.16.9",
        "1.21.1",
    );

    let mut auth    = OfflineAuth::new("Player123");
    let     profile = auth.authenticate(
        #[cfg(feature = "events")] None,
    ).await?;

    instance.launch(&profile, JavaDistribution::Temurin)
        .with_jvm_options()
            .set("Xmx", "4G")
            .set("XX:+UseG1GC", "")
            .done()
        .with_arguments()
            .set("width",  "1920")
            .set("height", "1080")
            .done()
        .run()
        .await?;

    if let Some(pid) = instance.get_pid() {
        println!("Running with PID {pid}");
    }
    Ok(())
}
```

## Error handling

```rust,ignore
pub enum InstallerError {
    DownloadFailed(String),
    VerificationFailed(String),
    ExtractionFailed(String),
    InvalidMetadata,
    NoPid,
    IOError(std::io::Error),
    // …
}
```

`InstanceError` is separate (manager-level), documented in
[instance-control.md](./instance-control.md).

## Related

- [How to use](./how-to-use.md) — short patterns
- [Installation](./installation.md) — the 8 buckets, SHA1, modpack
- [Instance lifecycle](./instance-lifecycle.md) — manager + console
- [Instance control](./instance-control.md) — PID / close / delete API
- [Arguments](./arguments.md) — placeholders, JVM / game args, token routing
- [Events](./events.md) — `LaunchEvent` + `ModloaderEvent`
- [Exports](./exports.md)
