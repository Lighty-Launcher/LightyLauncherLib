# lighty-event

Real-time event system for monitoring launcher operations and progress tracking.

## Overview

`lighty-event` provides a publish-subscribe event system for:
- **Real-time Progress Tracking** - Download, install, and extraction progress
- **Lifecycle Monitoring** - Authentication, launch, and instance management
- **Console Streaming** - Live game console output (stdout/stderr)
- **Multi-Subscriber Support** - Broadcast to UI, logs, and analytics simultaneously

## Quick Start

```toml
[dependencies]
lighty-event = "26.5.9"
```

```rust
use lighty_event::{EventBus, Event};

#[tokio::main]
async fn main() {
    // Create event bus
    let event_bus = EventBus::new(1000);
    let mut receiver = event_bus.subscribe();

    // Spawn event listener
    tokio::spawn(async move {
        while let Ok(event) = receiver.next().await {
            match event {
                Event::DownloadProgress(e) => {
                    let percent = (e.current * 100) / e.total;
                    println!("Download: {}%", percent);
                }
                Event::InstanceLaunched(e) => {
                    println!("Game launched: {} (PID: {})", e.instance_name, e.pid);
                }
                Event::ConsoleOutput(e) => {
                    print!("[{}] {}", e.stream, e.line);
                }
                _ => {}
            }
        }
    });

    // Use event_bus with launcher operations
    // auth.authenticate(Some(&event_bus)).await.unwrap();
    // version.launch(&profile, java).with_event_bus(&event_bus).run().await.unwrap();
}
```

## Event Categories

| Category | Description | Key Events |
|----------|-------------|------------|
| **AuthEvent** | Authentication flow | `AuthenticationStarted`, `AuthenticationSuccess`, `AuthenticationFailed` |
| **JavaEvent** | JRE management | `JavaDownloadProgress`, `JavaExtractionProgress`, `JavaAlreadyInstalled` |
| **LaunchEvent** | Game lifecycle | `InstallStarted`, `InstallProgress`, `Launched`, `ProcessExited` |
| **LoaderEvent** | Mod loaders | `FetchingData`, `DataFetched`, `ManifestCached`, `MergingLoaderData` |
| **CoreEvent** | System operations | `DownloadStarted`, `ExtractionProgress`, `VerificationCompleted` |
| **InstanceEvent** | Instance management | `InstanceLaunched`, `ConsoleOutput`, `InstanceExited`, `InstanceDeleted` |

## Documentation

📚 **[Complete Documentation](./docs)**

| Guide | Description |
|-------|-------------|
| [Overview](./docs/overview.md) | What this crate does |
| [How to use](./docs/how-to-use.md) | Subscribe / filter / fan-out patterns |
| [Event Reference](./docs/events.md) | Complete event catalog |
| [Architecture](./docs/architecture.md) | Event bus design and data flow |
| [Module System](./docs/modules.md) | Event module organization |
| [Exports](./docs/exports.md) | Public API surface |

## License

MIT

## Links

- **Main Package**: [lighty-launcher](https://crates.io/crates/lighty-launcher)
- **Repository**: [GitHub](https://github.com/Lighty-Launcher/LightyLauncherLib)
- **Documentation**: [docs.rs/lighty-event](https://docs.rs/lighty-event)
