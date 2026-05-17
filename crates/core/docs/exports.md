# Exports

## Overview

This document provides a complete reference of all exports from `lighty-core` and their re-exports in `lighty-launcher`.

## In `lighty_core`

### App State

```rust
use lighty_core::AppState;
use lighty_core::app_state::AppState; // Full path
```

### System Detection

```rust
use lighty_core::system::{
    // Constants
    OS,           // Current OS at compile-time
    ARCHITECTURE, // Current architecture at compile-time

    // Types
    OperatingSystem,
    Architecture,
};
```

### Download System

```rust
use lighty_core::download::{
    download_file,           // Download with progress callback
    download_file_untracked, // Download without tracking
};
```

### Extract System

```rust
use lighty_core::extract::{
    zip_extract,     // Extract ZIP archives
    tar_extract,     // Extract TAR archives
    tar_gz_extract,  // Extract TAR.GZ archives
};
```

### Hash Utilities

```rust
use lighty_core::hash::{
    // Async
    verify_file_sha1,           // Verify SHA1 (small files)
    verify_file_sha1_streaming, // Verify SHA1 (large files, streaming)

    // Sync
    calculate_file_sha1_sync,   // Calculate SHA1 (sync)
    verify_file_sha1_sync,      // Verify SHA1 (sync)

    // Bytes
    calculate_sha1_bytes,       // Calculate SHA1 of bytes → hex string
    calculate_sha1_bytes_raw,   // Calculate SHA1 of bytes → raw bytes
};
```

### HTTP Client

```rust
use lighty_core::hosts::HTTP_CLIENT; // Shared reqwest::Client
```

### Error Types

```rust
use lighty_core::errors::{
    // System
    SystemError,
    SystemResult,

    // Download
    DownloadError,
    DownloadResult,

    // Extract
    ExtractError,
    ExtractResult,

    // Hash
    HashError,
    HashResult,

    // AppState
    AppStateError,
    AppStateResult,
};

// AppStateError variants:
// - NotInitialized
// - AlreadyInitialized
// - MissingDir(&'static str)
```

**Or use direct paths**:
```rust
use lighty_core::{
    SystemError, SystemResult,
    DownloadError, DownloadResult,
    ExtractError, ExtractResult,
    HashError, HashResult,
    AppStateError, AppStateResult,
};
```

### Macros

All macros are automatically available when using `lighty_core`:

```rust
use lighty_core::{
    // File macros
    mkdir,              // Create directory if not exists
    join_and_mkdir,     // Join path and create directory
    join_and_mkdir_vec, // Join multiple paths and create directories

    // Logging macros (requires `tracing` feature)
    trace_debug,        // Debug log
    trace_info,         // Info log
    trace_warn,         // Warning log
    trace_error,        // Error log

    // Performance macro (requires `tracing` feature)
    time_it,            // Time an operation
};
```

## In `lighty_launcher` (Re-exports)

### From Main Crate

```rust
use lighty_launcher::core::{
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

## Usage Patterns

### Pattern 1: Direct Crate Import

```rust
use lighty_core::{
    AppState,
    system::{OS, ARCHITECTURE},
    download::download_file,
    hash::verify_file_sha1,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("MyLauncher")?;

    println!("OS: {:?}, Arch: {:?}", OS, ARCHITECTURE);

    Ok(())
}
```

### Pattern 2: Via Main Launcher Crate

```rust
use lighty_launcher::core::{
    AppState,
    system::{OS, ARCHITECTURE},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("MyLauncher")?;

    Ok(())
}
```

### Pattern 3: Prelude (if available)

```rust
// Note: lighty-core doesn't have a prelude, but lighty-launcher might
use lighty_launcher::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("MyLauncher")?;

    Ok(())
}
```

## Error Handling

### System Errors

```rust
use lighty_core::errors::{SystemError, SystemResult};
use lighty_core::system::OS;

fn get_platform_name() -> SystemResult<&'static str> {
    OS.get_vanilla_os()
}

match get_platform_name() {
    Ok(name) => println!("Platform: {}", name),
    Err(SystemError::UnsupportedOS) => {
        eprintln!("Unsupported operating system");
    }
    Err(SystemError::UnsupportedArchitecture) => {
        eprintln!("Unsupported architecture");
    }
}
```

### Download Errors

```rust
use lighty_core::download::download_file_untracked;
use lighty_core::errors::DownloadError;

match download_file_untracked("https://example.com/file", "output").await {
    Ok(_) => println!("Downloaded"),
    Err(e) => {
        eprintln!("Download failed: {}", e);
        // DownloadError implements Display
    }
}
```

### Extract Errors

```rust
use lighty_core::extract::zip_extract;
use lighty_core::errors::ExtractError;
use tokio::fs::File;
use tokio::io::BufReader;

let file = File::open("archive.zip").await?;
let reader = BufReader::new(file);

match zip_extract(reader, "output", None).await {
    Ok(_) => println!("Extracted"),
    Err(ExtractError::UnsupportedFormat(fmt)) => {
        eprintln!("Unsupported format: {}", fmt);
    }
    Err(ExtractError::IOError(e)) => {
        eprintln!("IO error: {}", e);
    }
    Err(ExtractError::PathTraversal { path }) => {
        eprintln!("Path traversal detected: {}", path);
    }
    Err(e) => {
        eprintln!("Extraction error: {}", e);
    }
}
```

### Hash Errors

```rust
use lighty_core::hash::verify_file_sha1;
use lighty_core::errors::HashError;
use std::path::Path;

match verify_file_sha1(Path::new("file.jar"), "expected-sha1").await {
    Ok(true) => println!("Hash valid"),
    Ok(false) => println!("Hash invalid"),
    Err(HashError::Io(e)) => {
        eprintln!("IO error: {}", e);
    }
    Err(HashError::Mismatch { expected, actual }) => {
        eprintln!("Hash mismatch:");
        eprintln!("  Expected: {}", expected);
        eprintln!("  Got:      {}", actual);
    }
}
```

### AppState Errors

```rust
use lighty_core::AppState;
use lighty_core::errors::AppStateError;

match AppState::init("MyOrg") {
    Ok(()) => println!("Initialized"),
    Err(AppStateError::AlreadyInitialized) => {
        eprintln!("AppState already initialized");
    }
    Err(e) => {
        eprintln!("Failed to initialize: {:?}", e);
    }
}
```

## Feature-Gated Exports

### Events Feature

```toml
[dependencies]
lighty-core = { version = "26.5.1", features = ["events"] }
```

When `events` feature is enabled:
- Extract functions accept `Option<&EventBus>` parameter
- `CoreEvent` types are used (from `lighty-event`)

```rust
#[cfg(feature = "events")]
use lighty_event::{EventBus, Event, CoreEvent};
use lighty_core::extract::zip_extract;

#[cfg(feature = "events")]
let event_bus = EventBus::new(1000);

#[cfg(feature = "events")]
zip_extract(reader, output, Some(&event_bus)).await?;

#[cfg(not(feature = "events"))]
zip_extract(reader, output).await?;
```

### Tracing Feature

```toml
[dependencies]
lighty-core = { version = "26.5.1", features = ["tracing"] }
```

When `tracing` feature is enabled:
- `trace_debug!`, `trace_info!`, `trace_warn!`, `trace_error!` macros emit logs
- `time_it!` macro emits performance metrics

When disabled:
- All macros compile to no-ops

## Module Structure

```
lighty_core
├── app_state
│   └── AppState
├── system
│   ├── OS
│   ├── ARCHITECTURE
│   ├── OperatingSystem
│   └── Architecture
├── download
│   ├── download_file
│   └── download_file_untracked
├── extract
│   ├── zip_extract
│   ├── tar_extract
│   └── tar_gz_extract
├── hash
│   ├── verify_file_sha1
│   ├── verify_file_sha1_streaming
│   ├── calculate_file_sha1_sync
│   ├── verify_file_sha1_sync
│   ├── calculate_sha1_bytes
│   └── calculate_sha1_bytes_raw
├── hosts
│   └── HTTP_CLIENT
├── macros
│   ├── mkdir!
│   ├── join_and_mkdir!
│   ├── join_and_mkdir_vec!
│   ├── trace_debug!
│   ├── trace_info!
│   ├── trace_warn!
│   ├── trace_error!
│   └── time_it!
└── errors
    ├── SystemError / SystemResult
    ├── DownloadError / DownloadResult
    ├── ExtractError / ExtractResult
    ├── HashError / HashResult
    └── AppStateError / AppStateResult
```

## Related Documentation

- [How to Use](./how-to-use.md) - Practical usage examples
- [Overview](./overview.md) - Architecture and design
- [Events](./events.md) - CoreEvent types
- [AppState](./app_state.md) - Application state guide
- [Download](./download.md) - Download system
- [Extract](./extract.md) - Extraction system
- [Hash](./hash.md) - Hash verification
- [System](./system.md) - Platform detection
- [Macros](./macros.md) - Macro usage
