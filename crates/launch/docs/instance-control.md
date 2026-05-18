# Instance control

`InstanceControl` is the public API for managing a running game:
inspect PIDs, terminate the process, delete the on-disk instance,
compute its disk-space breakdown.

For the underlying state machine (registration, console streaming,
exit detection), see [instance-lifecycle.md](./instance-lifecycle.md).

## The trait

```rust
#[allow(async_fn_in_trait)]
pub trait InstanceControl: VersionInfo {
    fn get_pid (&self) -> Option<u32>;
    fn get_pids(&self) -> Vec<u32>;

    async fn close_instance (&self, pid: u32) -> InstanceResult<()>;
    async fn delete_instance(&self)           -> InstanceResult<()>;

    fn size_of_instance(&self, version: &Version) -> InstanceSize;
}

impl<T: VersionInfo> InstanceControl for T {}
```

Auto-implemented for every `VersionInfo`. **The trait must be imported**
in scope to call the methods:

```rust
use lighty_launch::InstanceControl;     // required!
```

## Methods

### `get_pid()` / `get_pids()`

Look up the running PID(s) for this instance name in the global
[`InstanceManager`](./instance-lifecycle.md#instance-manager).

```rust
# use lighty_auth::UserProfile;
# use lighty_core::AppState;
# use lighty_launch::launch::Launch;
# use lighty_launch::errors::InstallerResult;
# use lighty_java::JavaDistribution;
# use lighty_loaders::types::Loader;
# use lighty_version::VersionBuilder;
use lighty_launch::InstanceControl;

# async fn run() -> InstallerResult<()> {
# AppState::init("MyLauncher").ok();
# let profile = UserProfile::offline("Player", "");
# let mut instance = VersionBuilder::new("game", Loader::Vanilla, "", "1.21.1");
# instance.launch(&profile, JavaDistribution::Temurin).run().await?;
match instance.get_pid() {
    Some(pid) => println!("running with PID {pid}"),
    None      => println!("not running"),
}

let all = instance.get_pids();
println!("all running PIDs: {all:?}");
# Ok(()) }
```

`get_pids()` matters when the same instance name has been launched
multiple times (it returns every PID still tracked, in insertion
order).

### `close_instance(pid)`

Removes the entry from the manager (so subsequent `get_pid()` calls
return `None`), then sends the OS signal:

- **Unix** — `kill -SIGTERM <pid>` (lets the JVM run its shutdown
  hooks; avoids losing unflushed world state).
- **Windows** — `taskkill /PID <pid> /F` (terminates the process tree).

```rust
# use lighty_auth::UserProfile;
# use lighty_core::AppState;
# use lighty_launch::launch::Launch;
# use lighty_launch::errors::InstallerResult;
# use lighty_java::JavaDistribution;
# use lighty_loaders::types::Loader;
# use lighty_version::VersionBuilder;
use lighty_launch::InstanceControl;

# async fn run() -> InstallerResult<()> {
# AppState::init("MyLauncher").ok();
# let profile = UserProfile::offline("Player", "");
# let mut instance = VersionBuilder::new("game", Loader::Vanilla, "", "1.21.1");
# instance.launch(&profile, JavaDistribution::Temurin).run().await?;
if let Some(pid) = instance.get_pid() {
    instance.close_instance(pid).await?;
}
# Ok(()) }
```

### `delete_instance()`

Removes the instance's game directory from disk. Errors if any PID
for this instance is still tracked — close all PIDs first:

```rust
# use lighty_auth::UserProfile;
# use lighty_core::AppState;
# use lighty_launch::errors::InstallerResult;
# use lighty_launch::InstanceControl;
# use lighty_loaders::types::Loader;
# use lighty_version::VersionBuilder;
# async fn run() -> InstallerResult<()> {
# AppState::init("MyLauncher").ok();
# let instance = VersionBuilder::new("game", Loader::Vanilla, "", "1.21.1");
for pid in instance.get_pids() {
    instance.close_instance(pid).await?;
}
instance.delete_instance().await?;
# Ok(()) }
```

**Deleted**: the entire game directory (saves, mods, configs,
libraries, assets, client JAR).
**Preserved**: Java installations (shared across instances).

### `size_of_instance(version)`

Returns an `InstanceSize` (per-component byte counts from the metadata).
Useful to render a "this will install N MB" screen before
`.launch().run()` actually downloads anything.

```rust
# use lighty_core::AppState;
# use lighty_launch::errors::InstallerResult;
# use lighty_launch::InstanceControl;
# use lighty_loaders::types::version_metadata::VersionMetaData;
# use lighty_loaders::types::Loader;
# use lighty_version::VersionBuilder;
# async fn run() -> InstallerResult<()> {
# AppState::init("MyLauncher").ok();
# let mut instance = VersionBuilder::new("game", Loader::Vanilla, "", "1.21.1");
let metadata = instance.get_metadata().await?;
if let VersionMetaData::Version(version) = metadata.as_ref() {
    let size = instance.size_of_instance(version);
    println!("libraries {}", size.libraries);
    println!("assets    {}", size.assets);
    println!("client    {}", size.client);
    println!("mods      {}", size.mods);
    println!("natives   {}", size.natives);
    println!("total     {} bytes", size.total);
}
# Ok(()) }
```

`InstanceSize` is defined in
[`lighty-loaders`](../../loaders/docs/exports.md); helper formatters
like `InstanceSize::format(bytes)` and `.total_gb()` are exposed
there.

## Error types

```rust
pub enum InstanceError {
    NotFound      { pid: u32 },
    StillRunning  { instance_name: String, pids: Vec<u32> },
    Io            (std::io::Error),
    DuplicatePid  { pid: u32, existing_instance: String },
}

pub type InstanceResult<T> = Result<T, InstanceError>;
```

| Variant | When |
|---|---|
| `NotFound` | `close_instance(pid)` called for an unknown PID |
| `StillRunning` | `delete_instance()` called while any PID is tracked |
| `Io` | Filesystem error during `delete_instance` |
| `DuplicatePid` | Race between two concurrent registrations, or OS PID reuse before `unregister_instance` fired |

```rust
use lighty_launch::errors::InstanceError;
# use lighty_launch::InstanceControl;
# use lighty_loaders::types::Loader;
# use lighty_version::VersionBuilder;
# async fn run() -> anyhow::Result<()> {
# let instance = VersionBuilder::new("game", Loader::Vanilla, "", "1.21.1");

match instance.delete_instance().await {
    Ok(_) => println!("deleted"),
    Err(InstanceError::StillRunning { instance_name, pids }) => {
        eprintln!("close {instance_name} (PIDs {pids:?}) first");
    }
    Err(e) => eprintln!("{e}"),
}
# Ok(()) }
```

## Patterns

### Wait for the game to actually exit

`close_instance` only sends the kill signal — the OS still takes a
moment to clean up. If you need to wait, subscribe to
`LaunchEvent::ProcessExited` (with `events` feature):

```rust
# #[cfg(feature = "events")]
# {
use lighty_event::{Event, LaunchEvent};
# use lighty_event::EventBus;
# async fn run(bus: &EventBus, pid: u32) {
let mut rx = bus.subscribe();
while let Ok(event) = rx.next().await {
    if let Event::Launch(LaunchEvent::ProcessExited { pid: p, .. }) = event {
        if p == pid { break; }
    }
}
# }
# }
```

### Graceful close + delete

```rust
# use lighty_launch::InstanceControl;
# use lighty_launch::errors::InstanceError;
# use lighty_loaders::types::Loader;
# use lighty_version::VersionBuilder;
# async fn run() -> anyhow::Result<()> {
# let instance = VersionBuilder::new("game", Loader::Vanilla, "", "1.21.1");
// 1. Close every running PID for this instance name.
for pid in instance.get_pids() {
    match instance.close_instance(pid).await {
        Ok(_)                                  => {}
        Err(InstanceError::NotFound { .. })    => {}  // already gone, fine
        Err(e)                                 => return Err(e.into()),
    }
}

// 2. Delete the on-disk instance.
instance.delete_instance().await?;
# Ok(()) }
```

## Related

- [Launch](./launch.md) — how PIDs land in the manager in the first place
- [Instance lifecycle](./instance-lifecycle.md) — the underlying state machine
- [How to use](./how-to-use.md), [Events](./events.md), [Exports](./exports.md)
