# Instance Control

## Overview

The instance control system provides process management for running Minecraft instances, including PID tracking, lifecycle management, console streaming, and instance termination.

## InstanceControl Trait

**CRITICAL**: You must import the trait to use instance management methods.

```rust
use lighty_launch::InstanceControl;  // Required!

// Now you can use instance management methods
if let Some(pid) = instance.get_pid() {
    println!("Running: {}", pid);
}
```

### Trait Definition

```rust
pub trait InstanceControl: VersionInfo {
    /// Get the first PID for this instance
    fn get_pid(&self) -> Option<u32>;

    /// Get all PIDs for this instance (supports multiple processes)
    fn get_pids(&self) -> Vec<u32>;

    /// Close an instance by PID
    async fn close_instance(&self, pid: u32) -> InstanceResult<()>;

    /// Delete an instance completely (must not be running)
    async fn delete_instance(&self) -> InstanceResult<()>;

    /// Calculate the size of an instance
    fn size_of_instance(&self, version: &Version) -> InstanceSize;
}
```

**Auto-implemented** for any type implementing `VersionInfo`.

## Instance Lifecycle

```
┌─────────────────────────────────────────────────────────────┐
│                    Instance Lifecycle                        │
└─────────────────────────────────────────────────────────────┘

1. Launch
   ├─> Spawn Java process
   ├─> Capture PID
   ├─> Register in InstanceManager
   └─> Emit InstanceLaunched event

2. Running
   ├─> Stream stdout/stderr
   ├─> Emit ConsoleOutput events
   └─> Track process state

3. Exit
   ├─> Process terminates
   ├─> Emit InstanceExited event
   ├─> Unregister from InstanceManager
   └─> Cleanup resources

4. Manual Close (Optional)
   ├─> User calls close_instance(pid)
   ├─> Send kill signal (SIGTERM/TASKKILL)
   ├─> Wait for termination
   └─> Unregister from InstanceManager
```

## Instance Manager

### Internal Structure

```rust
pub(crate) struct GameInstance {
    pub pid: u32,
    pub instance_name: String,
    pub version: String,
    pub username: String,
    pub game_dir: PathBuf,
    pub started_at: SystemTime,
}

pub(crate) struct InstanceManager {
    instances: RwLock<HashMap<u32, GameInstance>>,
}

pub(crate) static INSTANCE_MANAGER: Lazy<InstanceManager> = Lazy::new(InstanceManager::new);
```

**Global singleton**: `INSTANCE_MANAGER` tracks all running instances

### Registration

When a game launches:
```rust
let instance = GameInstance {
    pid: child.id().unwrap(),
    instance_name: "my-instance".to_string(),
    version: "1.21.1-fabric-0.16.9".to_string(),
    username: "Player123".to_string(),
    game_dir: PathBuf::from("/path/to/instance"),
    started_at: SystemTime::now(),
};

INSTANCE_MANAGER.register_instance(instance).await;
```

### Unregistration

When a game exits or is closed:
```rust
INSTANCE_MANAGER.unregister_instance(pid).await;
```

## Process Management

### Get PID

Get the first PID for an instance:
```rust
use lighty_launch::InstanceControl;

if let Some(pid) = instance.get_pid() {
    println!("Instance is running with PID: {}", pid);
} else {
    println!("Instance is not running");
}
```

**Implementation**:
```rust
fn get_pid(&self) -> Option<u32> {
    INSTANCE_MANAGER.get_pid(self.name())
}
```

### Get All PIDs

Get all PIDs for an instance (supports multiple processes):
```rust
use lighty_launch::InstanceControl;

let pids = instance.get_pids();
if pids.is_empty() {
    println!("No instances running");
} else {
    println!("Running instances: {:?}", pids);
}
```

**Use case**: Multiple instances with the same name running simultaneously

### Close Instance

Terminate a running instance:
```rust
use lighty_launch::InstanceControl;

if let Some(pid) = instance.get_pid() {
    match instance.close_instance(pid).await {
        Ok(_) => println!("Instance closed successfully"),
        Err(e) => eprintln!("Failed to close: {}", e),
    }
}
```

**Implementation**:
```rust
async fn close_instance(&self, pid: u32) -> InstanceResult<()> {
    INSTANCE_MANAGER.close_instance(pid).await
}
```

**Platform-specific kill**:

#### Windows
```rust
use std::process::Command;

let output = Command::new("taskkill")
    .args(&["/PID", &pid.to_string(), "/F"])
    .output()?;

if !output.status.success() {
    return Err(InstanceError::ProcessKillFailed);
}
```

#### Linux/macOS
```rust
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;

kill(Pid::from_raw(pid as i32), Signal::SIGTERM)?;
```

**Error handling**:
```rust
pub enum InstanceError {
    NotFound { pid: u32 },
    ProcessKillFailed,
    Io(std::io::Error),
}
```

### Delete Instance

Delete an instance completely (must not be running):
```rust
use lighty_launch::InstanceControl;

match instance.delete_instance().await {
    Ok(_) => println!("Instance deleted"),
    Err(InstanceError::InstanceRunning) => {
        eprintln!("Cannot delete: instance is running");
    }
    Err(e) => eprintln!("Delete failed: {}", e),
}
```

**Implementation**:
```rust
async fn delete_instance(&self) -> InstanceResult<()> {
    // Check if instance is running
    if INSTANCE_MANAGER.has_running_instances() {
        return Err(InstanceError::InstanceRunning);
    }

    // Delete game directory
    tokio::fs::remove_dir_all(self.game_dirs()).await?;

    #[cfg(feature = "events")]
    {
        EVENT_BUS.emit(Event::InstanceDeleted(InstanceDeletedEvent {
            instance_name: self.name().to_string(),
            timestamp: SystemTime::now(),
        }));
    }

    Ok(())
}
```

**Deletes**:
- Game directory (`{game_dir}/`)
- All saves, mods, configs
- Libraries, assets, client JAR

**Preserves**:
- Java installations (shared across instances)

## Console Streaming

### Console Handler

Asynchronous console streaming:
```rust
pub(crate) async fn handle_console_streams(
    pid: u32,
    instance_name: String,
    mut child: Child
) {
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    // Spawn stdout handler
    if let Some(stdout) = stdout {
        tokio::spawn(stream_stdout(pid, instance_name.clone(), stdout));
    }

    // Spawn stderr handler
    if let Some(stderr) = stderr {
        tokio::spawn(stream_stderr(pid, instance_name.clone(), stderr));
    }

    // Wait for process exit
    match child.wait().await {
        Ok(status) => {
            emit_exit_event(pid, instance_name.clone(), status.code());
        }
        Err(e) => {
            log_exit_error(pid, instance_name.clone(), e);
        }
    }

    // Cleanup
    INSTANCE_MANAGER.unregister_instance(pid).await;
}
```

### Stdout Streaming

```rust
async fn stream_stdout(pid: u32, instance_name: String, stdout: ChildStdout) {
    let reader = BufReader::new(stdout);
    let mut lines = reader.lines();

    while let Ok(Some(line)) = lines.next_line().await {
        #[cfg(feature = "events")]
        {
            EVENT_BUS.emit(Event::ConsoleOutput(ConsoleOutputEvent {
                pid,
                instance_name: instance_name.clone(),
                stream: ConsoleStream::Stdout,
                line,
                timestamp: SystemTime::now(),
            }));
        }

        #[cfg(not(feature = "events"))]
        println!("[{}] {}", pid, line);
    }
}
```

### Stderr Streaming

```rust
async fn stream_stderr(pid: u32, instance_name: String, stderr: ChildStderr) {
    let reader = BufReader::new(stderr);
    let mut lines = reader.lines();

    while let Ok(Some(line)) = lines.next_line().await {
        #[cfg(feature = "events")]
        {
            EVENT_BUS.emit(Event::ConsoleOutput(ConsoleOutputEvent {
                pid,
                instance_name: instance_name.clone(),
                stream: ConsoleStream::Stderr,
                line,
                timestamp: SystemTime::now(),
            }));
        }

        #[cfg(not(feature = "events"))]
        eprintln!("[{}] {}", pid, line);
    }
}
```

### Console Events

```rust
pub struct ConsoleOutputEvent {
    pub pid: u32,
    pub instance_name: String,
    pub stream: ConsoleStream,
    pub line: String,
    pub timestamp: SystemTime,
}

pub enum ConsoleStream {
    Stdout,
    Stderr,
}
```

**Example usage**:
```rust
use lighty_event::{EventBus, Event, ConsoleStream};

let event_bus = EventBus::new(1000);
let mut receiver = event_bus.subscribe();

tokio::spawn(async move {
    while let Ok(event) = receiver.recv().await {
        if let Event::ConsoleOutput(e) = event {
            match e.stream {
                ConsoleStream::Stdout => println!("[OUT] {}", e.line),
                ConsoleStream::Stderr => eprintln!("[ERR] {}", e.line),
            }
        }
    }
});
```

## Instance Size

Calculate the size of an instance:
```rust
use lighty_launch::InstanceControl;

let metadata = instance.get_metadata().await?;
let version = match metadata.as_ref() {
    VersionMetaData::Version(v) => v,
    _ => panic!("Invalid metadata"),
};

let size = instance.size_of_instance(version);

println!("Libraries: {:.2} MB", size.libraries_mb());
println!("Assets: {:.2} MB", size.assets_mb());
println!("Client: {:.2} MB", size.client_mb());
println!("Mods: {:.2} MB", size.mods_mb());
println!("Total: {:.2} GB", size.total_gb());
```

### InstanceSize Structure

```rust
pub struct InstanceSize {
    pub libraries: u64,    // bytes
    pub assets: u64,       // bytes
    pub client: u64,       // bytes
    pub mods: u64,         // bytes
    pub natives: u64,      // bytes
}

impl InstanceSize {
    pub fn libraries_mb(&self) -> f64 {
        self.libraries as f64 / 1_048_576.0
    }

    pub fn assets_mb(&self) -> f64 {
        self.assets as f64 / 1_048_576.0
    }

    pub fn client_mb(&self) -> f64 {
        self.client as f64 / 1_048_576.0
    }

    pub fn mods_mb(&self) -> f64 {
        self.mods as f64 / 1_048_576.0
    }

    pub fn natives_mb(&self) -> f64 {
        self.natives as f64 / 1_048_576.0
    }

    pub fn total_mb(&self) -> f64 {
        (self.libraries + self.assets + self.client + self.mods + self.natives) as f64 / 1_048_576.0
    }

    pub fn total_gb(&self) -> f64 {
        self.total_mb() / 1024.0
    }
}
```

**Implementation**:
```rust
fn size_of_instance(&self, version: &Version) -> InstanceSize {
    let libraries = version.libraries.iter()
        .filter_map(|lib| lib.size)
        .sum();

    let assets = version.assets.as_ref()
        .map(|a| a.objects.values().map(|obj| obj.size).sum())
        .unwrap_or(0);

    let client = version.client.as_ref()
        .and_then(|c| c.size)
        .unwrap_or(0);

    let mods = version.mods.as_ref()
        .map(|m| m.iter().filter_map(|mod_| mod_.size).sum())
        .unwrap_or(0);

    let natives = version.natives.as_ref()
        .map(|n| n.iter().filter_map(|nat| nat.size).sum())
        .unwrap_or(0);

    InstanceSize {
        libraries,
        assets,
        client,
        mods,
        natives,
    }
}
```

## Events

### Instance Lifecycle Events

```rust
pub struct InstanceLaunchedEvent {
    pub pid: u32,
    pub instance_name: String,
    pub version: String,
    pub username: String,
    pub timestamp: SystemTime,
}

pub struct InstanceExitedEvent {
    pub pid: u32,
    pub instance_name: String,
    pub exit_code: Option<i32>,
    pub timestamp: SystemTime,
}

pub struct InstanceDeletedEvent {
    pub instance_name: String,
    pub timestamp: SystemTime,
}
```

### Listening to Events

```rust
use lighty_event::{EventBus, Event};

let event_bus = EventBus::new(1000);
let mut receiver = event_bus.subscribe();

tokio::spawn(async move {
    while let Ok(event) = receiver.recv().await {
        match event {
            Event::InstanceLaunched(e) => {
                println!("Launched: {} (PID: {})", e.instance_name, e.pid);
            }
            Event::ConsoleOutput(e) => {
                println!("[{}] {}", e.pid, e.line);
            }
            Event::InstanceExited(e) => {
                println!("Exited: {} (code: {:?})", e.instance_name, e.exit_code);
            }
            Event::InstanceDeleted(e) => {
                println!("Deleted: {}", e.instance_name);
            }
            _ => {}
        }
    }
});
```

## Complete Examples

### Basic Instance Management

```rust
use lighty_core::AppState;
use lighty_launcher::prelude::*;
use lighty_auth::{offline::OfflineAuth, Authenticator};
use lighty_java::JavaDistribution;
use lighty_launch::InstanceControl;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("MyLauncher")?;

    let mut instance = VersionBuilder::new(
        "my-game",
        Loader::Vanilla,
        "",
        "1.21.1",
    );

    // Authenticate
    let mut auth = OfflineAuth::new("Player");
    let profile = auth.authenticate().await?;

    // Launch
    instance.launch(&profile, JavaDistribution::Temurin)
        .run()
        .await?;

    // Get PID
    if let Some(pid) = instance.get_pid() {
        println!("Game running with PID: {}", pid);

        // Wait a bit
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;

        // Close instance
        instance.close_instance(pid).await?;
        println!("Instance closed");
    }

    Ok(())
}
```

### Multiple Instance Tracking

```rust
use lighty_launch::InstanceControl;

// Launch multiple instances
let mut instance1 = VersionBuilder::new("game1", Loader::Vanilla, "", "1.21.1");
let mut instance2 = VersionBuilder::new("game2", Loader::Fabric, "0.16.9", "1.21.1");

let profile1 = auth1.authenticate().await?;
let profile2 = auth2.authenticate().await?;

instance1.launch(&profile1, JavaDistribution::Temurin).run().await?;
instance2.launch(&profile2, JavaDistribution::Temurin).run().await?;

// Get all running PIDs
let pids1 = instance1.get_pids();
let pids2 = instance2.get_pids();

println!("Instance 1 PIDs: {:?}", pids1);
println!("Instance 2 PIDs: {:?}", pids2);

// Close all instances
for pid in pids1 {
    instance1.close_instance(pid).await?;
}

for pid in pids2 {
    instance2.close_instance(pid).await?;
}
```

### Console Monitoring

```rust
use lighty_event::{EventBus, Event, ConsoleStream};
use lighty_launch::InstanceControl;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("MyLauncher")?;

    let event_bus = EventBus::new(1000);

    // Console monitor task
    let mut receiver = event_bus.subscribe();
    tokio::spawn(async move {
        while let Ok(event) = receiver.recv().await {
            match event {
                Event::ConsoleOutput(e) => {
                    let prefix = match e.stream {
                        ConsoleStream::Stdout => "[OUT]",
                        ConsoleStream::Stderr => "[ERR]",
                    };
                    println!("{} [{}] {}", prefix, e.pid, e.line);
                }
                Event::InstanceExited(e) => {
                    println!("Instance {} exited with code {:?}", e.pid, e.exit_code);
                }
                _ => {}
            }
        }
    });

    // Launch instance
    let mut instance = VersionBuilder::new("game", Loader::Vanilla, "", "1.21.1");
    let mut auth = OfflineAuth::new("Player");
    let profile = auth.authenticate(None).await?;

    instance.launch(&profile, JavaDistribution::Temurin)
        .with_event_bus(&event_bus)
        .run()
        .await?;

    // Keep running to monitor console
    tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;

    Ok(())
}
```

### Instance Size Calculation

```rust
use lighty_launch::InstanceControl;

let mut instance = VersionBuilder::new("game", Loader::Fabric, "0.16.9", "1.21.1");

// Get metadata
let metadata = instance.get_metadata().await?;
let version = match metadata.as_ref() {
    VersionMetaData::Version(v) => v,
    _ => return Err(anyhow::anyhow!("Invalid metadata")),
};

// Calculate size
let size = instance.size_of_instance(version);

println!("Instance Size Breakdown:");
println!("  Libraries: {:.2} MB ({} files)", size.libraries_mb(), version.libraries.len());
println!("  Assets:    {:.2} MB ({} files)", size.assets_mb(), version.assets.as_ref().map(|a| a.objects.len()).unwrap_or(0));
println!("  Client:    {:.2} MB", size.client_mb());
println!("  Mods:      {:.2} MB ({} files)", size.mods_mb(), version.mods.as_ref().map(|m| m.len()).unwrap_or(0));
println!("  Natives:   {:.2} MB ({} files)", size.natives_mb(), version.natives.as_ref().map(|n| n.len()).unwrap_or(0));
println!("  ─────────────────────────");
println!("  Total:     {:.2} GB", size.total_gb());
```

**Example output**:
```
Instance Size Breakdown:
  Libraries: 52.34 MB (187 files)
  Assets:    284.67 MB (5234 files)
  Client:    24.18 MB
  Mods:      42.89 MB (23 files)
  Natives:   8.12 MB (12 files)
  ─────────────────────────
  Total:     0.40 GB
```

## Error Handling

### InstanceError Types

```rust
pub enum InstanceError {
    /// Instance not found by PID
    NotFound { pid: u32 },

    /// Cannot delete while instance is running
    InstanceRunning,

    /// Failed to kill process
    ProcessKillFailed,

    /// I/O error
    Io(std::io::Error),
}
```

### Error Examples

```rust
use lighty_launch::InstanceControl;
use lighty_launch::errors::InstanceError;

// Close instance
match instance.close_instance(pid).await {
    Ok(_) => println!("Closed successfully"),
    Err(InstanceError::NotFound { pid }) => {
        eprintln!("No instance found with PID: {}", pid);
    }
    Err(InstanceError::ProcessKillFailed) => {
        eprintln!("Failed to kill process");
    }
    Err(e) => eprintln!("Error: {}", e),
}

// Delete instance
match instance.delete_instance().await {
    Ok(_) => println!("Deleted successfully"),
    Err(InstanceError::InstanceRunning) => {
        eprintln!("Cannot delete: instance is still running");
        // Close first
        if let Some(pid) = instance.get_pid() {
            instance.close_instance(pid).await?;
        }
        // Then delete
        instance.delete_instance().await?;
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## Best Practices

### 1. Always Import the Trait

```rust
use lighty_launch::InstanceControl;  // Required!
```

### 2. Check Before Closing

```rust
if let Some(pid) = instance.get_pid() {
    instance.close_instance(pid).await?;
} else {
    println!("Instance not running");
}
```

### 3. Close Before Deleting

```rust
// Close all running instances first
for pid in instance.get_pids() {
    instance.close_instance(pid).await?;
}

// Then delete
instance.delete_instance().await?;
```

### 4. Monitor Console with Events

```rust
#[cfg(feature = "events")]
{
    let event_bus = EventBus::new(1000);
    instance.launch(&profile, JavaDistribution::Temurin)
        .with_event_bus(&event_bus)
        .run()
        .await?;
}
```

### 5. Handle Errors Gracefully

```rust
match instance.close_instance(pid).await {
    Ok(_) => {}
    Err(InstanceError::NotFound { .. }) => {
        // Already closed, that's fine
    }
    Err(e) => return Err(e.into()),
}
```

## Related Documentation

- [Launch Process](./launch.md) - Complete launch flow
- [Events](./events.md) - Event system reference
- [How to Use](./how-to-use.md) - Practical examples
- [Exports](./exports.md) - Module exports
