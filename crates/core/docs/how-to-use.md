# How to Use lighty-core

## Basic Usage

### Step 1: Initialize AppState

```rust
use lighty_core::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("MyLauncher")?;

    // AppState exposes static accessors:
    // - data_dir()  -> game files, instances, versions
    // - cache_dir() -> java runtimes, temporary files
    // - config_dir() -> configuration files

    println!("Data: {}", AppState::data_dir().display());
    println!("Cache: {}", AppState::cache_dir().display());
    println!("Config: {}", AppState::config_dir().display());

    Ok(())
}
```

### Step 2: Use System Detection

```rust
use lighty_core::system::{OS, ARCHITECTURE, OperatingSystem, Architecture};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Constants for compile-time detection
    println!("Operating System: {:?}", OS);
    println!("Architecture: {:?}", ARCHITECTURE);

    // Platform-specific names
    let vanilla_os = OS.get_vanilla_os()?; // "windows", "linux", "osx"
    let adoptium_os = OS.get_adoptium_name()?; // For Java downloads
    let archive_type = OS.get_archive_type()?; // "zip" or "tar.gz"

    println!("Vanilla OS: {}", vanilla_os);
    println!("Archive type: {}", archive_type);

    // Architecture info
    let arch_bits = ARCHITECTURE.get_arch_bits()?; // "32" or "64"
    let simple_arch = ARCHITECTURE.get_simple_name()?;

    println!("Architecture: {} ({}bit)", simple_arch, arch_bits);

    Ok(())
}
```

## Download System

### Basic File Download

```rust
use lighty_core::download::download_file;
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Download with progress callback
    let data = download_file(
        "https://example.com/file.dat",
        |current, total| {
            let progress = (current as f64 / total as f64) * 100.0;
            println!("Download progress: {:.1}%", progress);
        }
    ).await?;

    // Save to file
    tokio::fs::write("output.dat", &data).await?;

    Ok(())
}
```

### Download Without Tracking

```rust
use lighty_core::download::download_file_untracked;
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Simple download without progress
    download_file_untracked(
        "https://example.com/file.dat",
        "output.dat"
    ).await?;

    println!("Download complete!");

    Ok(())
}
```

## Hash Verification

### Async SHA1 Verification

```rust
use lighty_core::hash::{verify_file_sha1, verify_file_sha1_streaming};
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let path = Path::new("myfile.jar");
    let expected_sha1 = "abc123def456...";

    // Small files: read entire file
    let is_valid = verify_file_sha1(path, expected_sha1).await?;
    println!("File valid: {}", is_valid);

    // Large files: streaming verification
    let is_valid_streaming = verify_file_sha1_streaming(path, expected_sha1).await?;
    println!("File valid (streaming): {}", is_valid_streaming);

    Ok(())
}
```

### Sync SHA1 Verification

```rust
use lighty_core::hash::{calculate_file_sha1_sync, verify_file_sha1_sync, calculate_sha1_bytes};
use std::path::Path;

fn main() -> anyhow::Result<()> {
    // Calculate hash
    let hash = calculate_file_sha1_sync(Path::new("file.jar"))?;
    println!("SHA1: {}", hash);

    // Verify hash
    let is_valid = verify_file_sha1_sync(Path::new("file.jar"), &hash)?;
    println!("Valid: {}", is_valid);

    // Hash arbitrary data
    let username_hash = calculate_sha1_bytes(b"Player123");
    println!("Username hash: {}", username_hash);

    Ok(())
}
```

## Archive Extraction

### Extract ZIP Archive

```rust
use lighty_core::extract::zip_extract;
use tokio::fs::File;
use tokio::io::BufReader;
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let archive_path = Path::new("archive.zip");
    let output_dir = Path::new("extracted");

    // Open archive
    let file = File::open(archive_path).await?;
    let reader = BufReader::new(file);

    // Extract (with optional event bus for progress tracking)
    #[cfg(feature = "events")]
    {
        use lighty_event::EventBus;
        let event_bus = EventBus::new(1000);
        zip_extract(reader, output_dir, Some(&event_bus)).await?;
    }

    #[cfg(not(feature = "events"))]
    {
        zip_extract(reader, output_dir).await?;
    }

    println!("Extraction complete!");

    Ok(())
}
```

**Security Features**:
- Path traversal protection
- Symlink/hardlink rejection
- Absolute path rejection
- File size limits (2GB max)
- Path sanitization

### Extract TAR Archives

```rust
use lighty_core::extract::{tar_extract, tar_gz_extract};
use tokio::fs::File;
use tokio::io::BufReader;
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Extract .tar
    let tar_file = File::open("archive.tar").await?;
    let tar_reader = BufReader::new(tar_file);

    #[cfg(feature = "events")]
    {
        use lighty_event::EventBus;
        let event_bus = EventBus::new(1000);
        tar_extract(tar_reader, Path::new("output"), Some(&event_bus)).await?;
    }

    #[cfg(not(feature = "events"))]
    {
        tar_extract(tar_reader, Path::new("output")).await?;
    }

    // Extract .tar.gz
    let gz_file = File::open("archive.tar.gz").await?;
    let gz_reader = BufReader::new(gz_file);

    #[cfg(feature = "events")]
    {
        use lighty_event::EventBus;
        let event_bus = EventBus::new(1000);
        tar_gz_extract(gz_reader, Path::new("output_gz"), Some(&event_bus)).await?;
    }

    #[cfg(not(feature = "events"))]
    {
        tar_gz_extract(gz_reader, Path::new("output_gz")).await?;
    }

    Ok(())
}
```

## Macros

### Logging Macros

```rust
use lighty_core::{trace_debug, trace_info, trace_warn, trace_error};

#[tokio::main]
async fn main() {
    trace_debug!("Debug message: {}", "details");
    trace_info!("Operation started");
    trace_warn!("Warning: {}", "something happened");
    trace_error!("Error occurred: {}", "error details");
}
```

**Note**: Logging macros require the `tracing` feature. Without it, they compile to no-ops.

### Directory Creation Macros

```rust
use lighty_core::{mkdir, join_and_mkdir};
use std::path::Path;

#[tokio::main]
async fn main() {
    let base = Path::new("/tmp/launcher");

    // Create directory if it doesn't exist
    mkdir!(base);

    // Join path and create directory
    let instances_dir = join_and_mkdir!(base, "instances");
    println!("Created: {}", instances_dir.display());

    // Chain multiple joins
    use lighty_core::join_and_mkdir_vec;
    let world_dir = join_and_mkdir_vec!(base, &["instances", "my-instance", "saves", "world1"]);
    println!("Created: {}", world_dir.display());
}
```

## HTTP Client

The HTTP client is shared across all operations:

```rust
use lighty_core::hosts::HTTP_CLIENT;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Use the shared HTTP client
    let response = HTTP_CLIENT
        .get("https://api.example.com/data")
        .send()
        .await?;

    let data: serde_json::Value = response.json().await?;
    println!("Response: {:?}", data);

    Ok(())
}
```

**Benefits**:
- Connection pooling
- Shared across entire application
- Thread-safe
- Configured for optimal performance

## With Events

```rust
use lighty_core::{AppState, extract::zip_extract};
use lighty_event::{EventBus, Event, CoreEvent};
use tokio::fs::File;
use tokio::io::BufReader;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("MyLauncher")?;

    // Create event bus
    let event_bus = EventBus::new(1000);
    let mut receiver = event_bus.subscribe();

    // Spawn event listener
    tokio::spawn(async move {
        while let Ok(event) = receiver.next().await {
            match event {
                Event::Core(CoreEvent::ExtractionStarted { archive_type, file_count, destination }) => {
                    println!("Starting {} extraction: {} files to {}", archive_type, file_count, destination);
                }
                Event::Core(CoreEvent::FileExtracted { file_name, index, total }) => {
                    let progress = (index as f64 / total as f64) * 100.0;
                    println!("[{:.1}%] Extracted: {}", progress, file_name);
                }
                Event::Core(CoreEvent::ExtractionComplete { file_count }) => {
                    println!("Extraction complete! {} files extracted", file_count);
                }
                _ => {}
            }
        }
    });

    // Extract archive with events
    let file = File::open("archive.zip").await?;
    let reader = BufReader::new(file);
    zip_extract(reader, "output", Some(&event_bus)).await?;

    Ok(())
}
```

## Error Handling

```rust
use lighty_core::errors::{DownloadError, ExtractError, HashError, AppStateError};

#[tokio::main]
async fn main() {
    // AppState errors
    match AppState::init("MyLauncher") {
        Ok(()) => println!("Initialized"),
        Err(AppStateError::AlreadyInitialized) => {
            eprintln!("Already initialized");
        }
        Err(e) => {
            eprintln!("Failed to initialize: {:?}", e);
        }
    }

    // Download errors
    use lighty_core::download::download_file_untracked;
    match download_file_untracked("https://bad-url.com/file", "output").await {
        Ok(_) => println!("Downloaded"),
        Err(e) => eprintln!("Download error: {}", e),
    }

    // Hash errors
    use lighty_core::hash::verify_file_sha1;
    use std::path::Path;
    match verify_file_sha1(Path::new("file.jar"), "expected").await {
        Ok(true) => println!("Hash matches"),
        Ok(false) => println!("Hash mismatch"),
        Err(HashError::Io(e)) => eprintln!("IO error: {}", e),
        Err(HashError::Mismatch { expected, actual }) => {
            eprintln!("Hash mismatch: expected {}, got {}", expected, actual);
        }
    }
}
```

## Feature Flags

```toml
[dependencies]
lighty-core = { version = "26.5.1", features = ["events", "tracing"] }
```

Available features:
- `events` - Enables CoreEvent emission (requires lighty-event)
- `tracing` - Enables logging macros

## Exports

**In lighty_core**:
```rust
use lighty_core::{
    // App State
    AppState,

    // System
    system::{OS, ARCHITECTURE, OperatingSystem, Architecture},

    // Download
    download::{download_file, download_file_untracked},

    // Extract
    extract::{zip_extract, tar_extract, tar_gz_extract},

    // Hash
    hash::{
        verify_file_sha1, verify_file_sha1_streaming,
        calculate_file_sha1_sync, verify_file_sha1_sync,
        calculate_sha1_bytes, calculate_sha1_bytes_raw,
    },

    // HTTP Client
    hosts::HTTP_CLIENT,

    // Macros (automatically in scope when using crate)
    // mkdir!, join_and_mkdir!, join_and_mkdir_vec!
    // trace_debug!, trace_info!, trace_warn!, trace_error!

    // Errors
    errors::{
        SystemError, SystemResult,
        DownloadError, DownloadResult,
        ExtractError, ExtractResult,
        HashError, HashResult,
        AppStateError, AppStateResult,
    },
};
```

**In lighty_launcher** (re-exports):
```rust
use lighty_launcher::core::{
    AppState,
    system::{OS, ARCHITECTURE},
    // ... other types
};
```

## Related Documentation

- [Overview](./overview.md) - Architecture and design
- [Events](./events.md) - CoreEvent types
- [Exports](./exports.md) - Complete export reference
- [AppState](./app_state.md) - Detailed AppState guide
- [Download](./download.md) - Download system details
- [Extract](./extract.md) - Extraction system details

## Related Crates

- **[lighty-event](../../event/README.md)** - Event system
- **[lighty-loaders](../../loaders/README.md)** - Uses AppState and system detection
- **[lighty-java](../../java/README.md)** - Uses download and extract
- **[lighty-launch](../../launch/README.md)** - Uses AppState
