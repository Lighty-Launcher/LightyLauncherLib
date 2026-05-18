# Exports

Public surface of the `lighty-launch` crate.

## Module layout

```
lighty_launch
├── launch
│   ├── Launch              (trait — adds .launch(profile, java))
│   ├── LaunchBuilder
│   └── LaunchConfig
├── installer
│   └── Installer           (trait — async fn install(&self, &Version, …))
├── instance
│   ├── InstanceControl     (trait — also re-exported at the crate root)
│   ├── InstanceError
│   └── InstanceResult
├── arguments
│   ├── Arguments           (trait)
│   ├── KEY_*               (placeholder key constants — see arguments.md)
│   └── …
└── errors
    ├── InstallerError
    ├── InstallerResult
    ├── InstanceError
    └── InstanceResult
```

## Crate root re-exports

```rust,no_run
use lighty_launch::{
    LaunchBuilder,
    LaunchConfig,
    Installer,
    InstanceControl,
    InstanceError,
    InstanceResult,
};
```

`InstanceControl` lives under `lighty_launch::instance::InstanceControl`
and is re-exported at the root for convenience — **import the trait**
to use its methods (`get_pid`, `close_instance`, `delete_instance`,
`size_of_instance`).

## Per-module imports

```rust,no_run
// The fluent .launch().run() entry point.
use lighty_launch::launch::{Launch, LaunchBuilder, LaunchConfig};

// The install pipeline trait.
use lighty_launch::installer::Installer;

// The instance management trait + errors.
use lighty_launch::instance::{InstanceControl, InstanceError, InstanceResult};

// The argv builder trait + placeholder key constants.
use lighty_launch::arguments::{Arguments, KEY_LAUNCHER_NAME, KEY_GAME_DIRECTORY};

// Pipeline-wide errors.
use lighty_launch::errors::{InstallerError, InstallerResult};
```

## Public types — at a glance

### `LaunchBuilder<'a, T>`

```rust,ignore
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

Full API and overrides reference: [arguments.md](./arguments.md).

### `LaunchConfig`

Shared launch configuration (currently username / uuid / java
distribution). Has a `Default` impl. Used by higher-level helpers
that want a one-shot config object rather than the builder.

```rust,ignore
pub struct LaunchConfig {
    pub username: String,
    pub uuid: String,
    pub java_distribution: JavaDistribution,
}
```

### `Launch` trait

Blanket-implemented for every type that satisfies the pipeline's
bounds. Full doc: [launch.md](./launch.md#the-launch-trait).

### `Installer` trait

```rust,ignore
pub trait Installer {
    fn install(
        &self,
        builder: &Version,
        #[cfg(feature = "events")] event_bus: Option<&EventBus>,
    ) -> impl Future<Output = InstallerResult<()>> + Send;
}
```

Auto-implemented for `T: VersionInfo<LoaderType = Loader> + WithMods`.
Detail: [installation.md](./installation.md).

### `Arguments` trait

```rust,ignore
pub trait Arguments {
    fn build_arguments(
        &self,
        builder: &Version,
        profile: Option<&UserProfile>,
        arg_overrides:  &HashMap<String, String>,
        arg_removals:   &HashSet<String>,
        jvm_overrides:  &HashMap<String, String>,
        jvm_removals:   &HashSet<String>,
        raw_args:       &[String],
    ) -> Vec<String>;
}
```

Auto-implemented for every `VersionInfo`. Detail:
[arguments.md](./arguments.md).

### `InstanceControl` trait

```rust,ignore
pub trait InstanceControl: VersionInfo {
    fn get_pid (&self) -> Option<u32>;
    fn get_pids(&self) -> Vec<u32>;
    async fn close_instance (&self, pid: u32) -> InstanceResult<()>;
    async fn delete_instance(&self)           -> InstanceResult<()>;
    fn size_of_instance(&self, version: &Version) -> InstanceSize;
}
```

Detail: [instance-control.md](./instance-control.md) (API),
[instance-lifecycle.md](./instance-lifecycle.md) (state machine).

### Errors

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

pub enum InstanceError {
    NotFound      { pid: u32 },
    StillRunning  { instance_name: String, pids: Vec<u32> },
    Io            (std::io::Error),
    DuplicatePid  { pid: u32, existing_instance: String },
}
```

`InstallerResult<T>` and `InstanceResult<T>` are the matching type
aliases.

### Placeholder key constants

The `lighty_launch::arguments` module exposes one `KEY_*` constant per
launch placeholder (`KEY_LAUNCHER_NAME`, `KEY_GAME_DIRECTORY`,
`KEY_AUTH_PLAYER_NAME`, …) so `.set(...)` calls aren't stringly typed.
Full table: [arguments.md → placeholder key constants](./arguments.md#placeholder-key-constants).

## Cargo features

| Feature | Adds |
|---|---|
| `events` | `LaunchEvent` + `ModloaderEvent` emission, `with_event_bus(...)` on `LaunchBuilder`, console streamed via events |
| `keyring` | Lets the argv builder read `--accessToken` from a `TokenHandle`. Forwards to `lighty-auth/keyring`. |
| `forge` | Forge install processors (1.13+) **and** legacy 1.7.10 – 1.12.2 universal-JAR extraction. Single switch for both eras. |
| `neoforge` | NeoForge install processors |
| `fabric`, `quilt` | Loader support (no extra install step) |
| `modrinth`, `curseforge` | User-mod resolver + matching modpack format parser |
| `tracing` | Provider-level `tracing` logs |

Enable from the umbrella crate (e.g. `lighty-launcher/keyring`) for
forwarded activation across `lighty-launch` and `lighty-auth`.

## Related

- [Overview](./overview.md), [How to use](./how-to-use.md)
- [Launch](./launch.md), [Installation](./installation.md)
- [Instance control](./instance-control.md), [Instance lifecycle](./instance-lifecycle.md)
- [Arguments](./arguments.md), [Events](./events.md)
