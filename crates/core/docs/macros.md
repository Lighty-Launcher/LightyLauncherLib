# Logging Macros

## Overview

Unified tracing macros that work with or without the `tracing` feature.

## Available Macros

```rust
trace_debug!(...)   // Debug-level messages
trace_info!(...)    // Informational messages
trace_warn!(...)    // Warning messages
trace_error!(...)   // Error messages
```

## Usage

### Basic Logging
```rust
use lighty_core::{trace_info, trace_error};

fn process_file(path: &str)  {
    trace_info!("Processing file: {}", path);

    match std::fs::read(path) {
        Ok(data) => {
            trace_info!("File read successfully: {} bytes", data.len());
            Ok(())
        }
        Err(e) => {
            trace_error!("Failed to read file: {}", e);
            Err(e.into())
        }
    }
}
```

### Structured Logging (with tracing feature)
```rust
trace_info!(
    file = %path,
    size = data.len(),
    "File processed successfully"
);
```

## Feature Flags

### With `tracing` Feature
```toml
[dependencies]
lighty-core = { version = "26.5.1", features = ["tracing"] }
```

Macros expand to `tracing::*!` calls with full structured logging support.

### Without `tracing` Feature
```toml
[dependencies]
lighty-core = "26.5.1"
```

Macros expand to no-ops (zero runtime cost).

## Best Practices

### 1. Use Appropriate Levels
```rust
trace_debug!("Cache hit for key: {}", key);        // Debug info
trace_info!("Download started: {}", url);           // User-facing info
trace_warn!("Retry attempt {}/3", attempt);         // Recoverable issues
trace_error!("Fatal error: {}", error);             // Critical failures
```

### 2. Include Context
```rust
trace_info!(
    version = %mc_version,
    loader = %loader_type,
    "Launching Minecraft"
);
```

### 3. Don't Log Sensitive Data
```rust
// Bad
trace_info!("Access token: {}", token);

// Good
trace_info!("User authenticated successfully");
```

## See Also

- [Examples](./examples.md)
- [tracing documentation](https://docs.rs/tracing)
