# How to use `lighty-launch`

The minimal flow is four steps: init `AppState`, build an instance,
authenticate, call `.launch().run()`.

```rust,no_run
use lighty_auth::{offline::OfflineAuth, Authenticator};
use lighty_core::AppState;
use lighty_java::JavaDistribution;
use lighty_launch::launch::Launch;
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

    let mut auth    = OfflineAuth::new("Player");
    let     profile = auth.authenticate(
        #[cfg(feature = "events")] None,
    ).await?;

    instance
        .launch(&profile, JavaDistribution::Temurin)
        .run()
        .await?;
    Ok(())
}
```

The four parts have canonical docs elsewhere — this page just shows
how they fit together. For more `VersionBuilder` patterns see
[`crates/version/docs/how-to-use.md`](../../version/docs/how-to-use.md);
for `AppState` paths see
[`crates/core/docs/app_state.md`](../../core/docs/app_state.md);
for `OfflineAuth` / `MicrosoftAuth` / `AzuriomAuth` see
[`crates/auth/docs/how-to-use.md`](../../auth/docs/how-to-use.md).

## Tuning JVM and game args

`.launch(...)` returns a [`LaunchBuilder`](./launch.md#launchbuilder-api)
exposing two sub-builders. The full placeholder list and option
catalogue live in [arguments.md](./arguments.md); the common knobs:

```rust,no_run
# use lighty_auth::UserProfile;
# use lighty_core::AppState;
# use lighty_java::JavaDistribution;
# use lighty_launch::errors::InstallerResult;
# use lighty_launch::launch::Launch;
# use lighty_loaders::types::Loader;
# use lighty_version::VersionBuilder;
# async fn run() -> InstallerResult<()> {
# AppState::init("MyLauncher").ok();
# let profile = UserProfile::offline("Player", "");
# let mut instance = VersionBuilder::new("inst", Loader::Vanilla, "", "1.21.1");
instance
    .launch(&profile, JavaDistribution::Temurin)
    .with_jvm_options()
        .set("Xmx", "4G")               // -Xmx4G
        .set("Xms", "2G")               // -Xms2G
        .set("XX:+UseG1GC", "")         // -XX:+UseG1GC
        .done()
    .with_arguments()
        .set("width",  "1920")          // --width 1920
        .set("height", "1080")          // --height 1080
        .done()
    .run()
    .await?;
# Ok(()) }
```

`set(key, "")` for value-less flags. `.remove(key)` strips an option
even if the loader metadata injected it. Custom keys default to
`--key value` form unless they're a known launch placeholder constant
(see [arguments.md](./arguments.md#standard-placeholders)).

## Tracking progress with events

Enable the `events` feature and pass an `EventBus` via
`.with_event_bus(&bus)`. The launch crate emits two families
(`LaunchEvent` for install / spawn / process I/O,
`ModloaderEvent` for mod resolution / modpack / resource-shader-datapack
buckets) — full catalogue and example listener in
[events.md](./events.md).

```rust,no_run
# #[cfg(feature = "events")]
# {
use lighty_event::{Event, EventBus, LaunchEvent};
use lighty_launch::launch::Launch;
# use lighty_auth::UserProfile;
# use lighty_core::AppState;
# use lighty_java::JavaDistribution;
# use lighty_launch::errors::InstallerResult;
# use lighty_loaders::types::Loader;
# use lighty_version::VersionBuilder;
# async fn run() -> InstallerResult<()> {
# AppState::init("MyLauncher").ok();
# let profile = UserProfile::offline("Player", "");
# let mut instance = VersionBuilder::new("inst", Loader::Vanilla, "", "1.21.1");
let bus = EventBus::new(1000);
let mut rx = bus.subscribe();

tokio::spawn(async move {
    while let Ok(event) = rx.next().await {
        if let Event::Launch(LaunchEvent::ProcessOutput { pid, line, .. }) = event {
            print!("[{pid}] {line}");
        }
    }
});

instance.launch(&profile, JavaDistribution::Temurin)
    .with_event_bus(&bus)
    .run()
    .await?;
# Ok(()) }
# }
```

## Inspect or close the running game

`InstanceControl` is auto-implemented for every `VersionInfo`. **You
must import the trait** to use its methods:

```rust,no_run
# use lighty_auth::UserProfile;
# use lighty_core::AppState;
# use lighty_java::JavaDistribution;
# use lighty_launch::errors::InstallerResult;
# use lighty_launch::launch::Launch;
# use lighty_loaders::types::Loader;
# use lighty_version::VersionBuilder;
use lighty_launch::InstanceControl;

# async fn run() -> InstallerResult<()> {
# AppState::init("MyLauncher").ok();
# let profile = UserProfile::offline("Player", "");
# let mut instance = VersionBuilder::new("inst", Loader::Vanilla, "", "1.21.1");
# instance.launch(&profile, JavaDistribution::Temurin).run().await?;
if let Some(pid) = instance.get_pid() {
    println!("running with PID {pid}");
    // SIGTERM on Unix, taskkill /F on Windows. JVM runs its shutdown hooks.
    instance.close_instance(pid).await?;
}

// Delete the instance from disk. Errors if any PID is still tracked.
// instance.delete_instance().await?;
# Ok(()) }
```

Disk-size breakdown (libraries / mods / assets / client / natives) is
exposed by `instance.size_of_instance(&version)` — see
[instance-control.md](./instance-control.md) for the full API.

## Installation without launch

`Installer` is also a standalone trait. The runner calls it for you,
but you can drive it directly for "download now, play later" flows:

```rust,no_run
# use lighty_core::AppState;
# use lighty_launch::errors::InstallerResult;
# use lighty_loaders::types::version_metadata::VersionMetaData;
# use lighty_loaders::types::Loader;
# use lighty_version::VersionBuilder;
use lighty_launch::installer::Installer;

# async fn run() -> InstallerResult<()> {
# AppState::init("MyLauncher").ok();
# let mut instance = VersionBuilder::new("inst", Loader::Vanilla, "", "1.21.1");
let metadata = instance.get_metadata().await?;
if let VersionMetaData::Version(v) = metadata.as_ref() {
    instance.install(v, #[cfg(feature = "events")] None).await?;
}
# Ok(()) }
```

See [installation.md](./installation.md) for the pipeline detail.

## Error handling

Two error types — `InstallerError` for everything install / launch
pipeline related, `InstanceError` for manager-level operations
(`close`, `delete`):

```rust,no_run
use lighty_launch::errors::{InstallerError, InstanceError};
# use lighty_auth::UserProfile;
# use lighty_core::AppState;
# use lighty_java::JavaDistribution;
# use lighty_launch::launch::Launch;
# use lighty_launch::InstanceControl;
# use lighty_loaders::types::Loader;
# use lighty_version::VersionBuilder;
# async fn run() -> anyhow::Result<()> {
# AppState::init("MyLauncher").ok();
# let profile = UserProfile::offline("Player", "");
# let mut instance = VersionBuilder::new("inst", Loader::Vanilla, "", "1.21.1");

match instance.launch(&profile, JavaDistribution::Temurin).run().await {
    Ok(_)                                       => println!("launched"),
    Err(InstallerError::DownloadFailed(msg))    => eprintln!("download: {msg}"),
    Err(InstallerError::NoPid)                  => eprintln!("spawn succeeded but no PID"),
    Err(e)                                      => eprintln!("{e}"),
}

match instance.delete_instance().await {
    Ok(_)                                                     => println!("deleted"),
    Err(InstanceError::StillRunning { instance_name, pids })  => {
        eprintln!("close PIDs first: {instance_name} {pids:?}");
    }
    Err(e)                                                    => eprintln!("{e}"),
}
# Ok(()) }
```

## Related

- [Overview](./overview.md), [Launch](./launch.md)
- [Installation](./installation.md), [Instance lifecycle](./instance-lifecycle.md)
- [Instance control](./instance-control.md), [Arguments](./arguments.md)
- [Events](./events.md), [Exports](./exports.md)
- Auth: [`crates/auth/docs/how-to-use.md`](../../auth/docs/how-to-use.md)
- `VersionBuilder`: [`crates/version/docs/how-to-use.md`](../../version/docs/how-to-use.md)
- Loaders: [`crates/loaders/docs/overview.md`](../../loaders/docs/overview.md)
