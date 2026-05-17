# Exports Reference

Complete reference of all exports in `lighty-loaders` and their re-exports in `lighty-launcher`.

## In lighty_loaders

### Types

```rust
use lighty_loaders::types::{
    // Core traits
    VersionInfo,
    LoaderExtensions,

    // Loader enum
    Loader,

    // Instance size calculation
    InstanceSize,

    // Version metadata structures
    version_metadata::{
        Version,
        VersionMetaData,
        Library,
        Asset,
        AssetIndex,
        Arguments,
        MainClass,
        Mods,
        Native,
    },
};
```

### Loaders Modules (feature-gated)

```rust
use lighty_loaders::loaders::{
    vanilla,    // feature = "vanilla"
    fabric,     // feature = "fabric"
    quilt,      // feature = "quilt"
    neoforge,   // feature = "neoforge"
    forge,      // feature = "forge"
    lighty_updater,  // feature = "lighty_updater"
    optifine,   // requires "vanilla" feature
};
```

Each loader module contains:
- Query enum (e.g., `vanilla::VanillaQuery`)
- Metadata types (e.g., `vanilla::VanillaMetaData`)
- Query implementation
- Repository singleton (e.g., `vanilla::VANILLA`)

### Utils

```rust
use lighty_loaders::utils::{
    cache,      // Cache implementation
    error,      // QueryError and Result types
    manifest,   // ManifestRepository
    query,      // Query trait
};
```

#### error

```rust
use lighty_core::{
    QueryError,
    Result,  // = std::result::Result<T, QueryError>
};

// QueryError variants:
// - NetworkError(String)
// - ParseError(String)
// - NotFound(String)
// - InvalidVersion(String)
// - CacheError(String)
// - UnsupportedLoader(String)
```

#### query

```rust
use lighty_loaders::utils::query::{
    Query,      // Main trait for implementing loaders
    QueryKey,   // Cache key type
    Result,     // Query result type
};
```

#### manifest

```rust
use lighty_loaders::utils::manifest::ManifestRepository;

// Used internally by loader implementations
```

#### cache

```rust
use lighty_loaders::utils::cache::{
    Cache,
    CachedData,
};

// Used internally for caching
```

## In lighty_launcher

### Via lighty_launcher::loaders

```rust
use lighty_launcher::loaders::{
    // Types
    Loader,
    VersionInfo,
    LoaderExtensions,
    InstanceSize,

    // Version metadata (full path)
    version_metadata::{
        Version,
        VersionMetaData,
        Library,
        // ... (same as lighty_loaders::types::version_metadata)
    },

    // Loaders modules
    vanilla,
    fabric,
    quilt,
    neoforge,
    forge,
    lighty_updater,
    optifine,

    // Utils
    cache,
    error,
    manifest,
    query,
};
```

### Via lighty_launcher::prelude

```rust
use lighty_launcher::prelude::*;

// Includes:
// - Loader
// - VersionInfo
// - LoaderExtensions
// - InstanceSize
```

### Root re-exports

```rust
use lighty_launcher::Loader;  // Direct access
```

## Usage Patterns

### Pattern 1: Use prelude (recommended)

```rust
use lighty_launcher::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("MyLauncher")?;

    let instance = VersionBuilder::new("name", Loader::Vanilla, "", "1.21.1");
    let metadata = instance.get_metadata().await?;

    Ok(())
}
```

### Pattern 2: Explicit imports

```rust
use lighty_launcher::loaders::{Loader, VersionInfo, LoaderExtensions};
use lighty_launcher::version::VersionBuilder;
use lighty_launcher::core::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("MyLauncher")?;

    let instance = VersionBuilder::new("name", Loader::Vanilla, "", "1.21.1");
    let metadata = instance.get_metadata().await?;

    Ok(())
}
```

### Pattern 3: Direct crate import (advanced)

```rust
use lighty_loaders::types::{Loader, VersionInfo, LoaderExtensions};
use lighty_version::VersionBuilder;
use lighty_core::app_state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("MyLauncher")?;

    let instance = VersionBuilder::new("name", Loader::Vanilla, "", "1.21.1");
    let metadata = instance.get_metadata().await?;

    Ok(())
}
```

## Error Handling

### QueryError

**Not re-exported in `lighty_launcher`** - use full path:

```rust
use lighty_core::QueryError;

match instance.get_metadata().await {
    Ok(metadata) => { /* ... */ }
    Err(QueryError::NetworkError(e)) => eprintln!("Network: {}", e),
    Err(QueryError::NotFound(v)) => eprintln!("Not found: {}", v),
    Err(QueryError::ParseError(e)) => eprintln!("Parse: {}", e),
    Err(QueryError::UnsupportedLoader(l)) => eprintln!("Unsupported: {}", l),
    Err(e) => eprintln!("Error: {:?}", e),
}
```

Or use `anyhow::Result`:

```rust
use anyhow::Result;

async fn fetch_metadata() -> Result<()> {
    let metadata = instance.get_metadata().await?;  // Automatic conversion
    Ok(())
}
```

## Events

**In lighty_event**:

```rust
use lighty_event::{Event, LoaderEvent};

// LoaderEvent variants:
// - FetchingData { loader, minecraft_version, loader_version }
// - DataFetched { loader, minecraft_version, loader_version }
// - ManifestNotFound { loader, minecraft_version, loader_version, error }
// - ManifestCached { loader }
// - MergingLoaderData { base_loader, overlay_loader }
// - DataMerged { base_loader, overlay_loader }
```

**In lighty_launcher**:

```rust
use lighty_launcher::event::{Event, LoaderEvent};
// or
use lighty_launcher::prelude::*;  // includes Event, LoaderEvent
```

See [Events](./events.md) for detailed event documentation.

## Feature Flags

Enable specific loaders in `Cargo.toml`:

```toml
[dependencies]
lighty-launcher = { version = "26.5.1", features = ["all-loaders"] }
# or
lighty-loaders = { version = "26.5.1", features = ["vanilla", "fabric", "quilt"] }
```

**Available features**:
- `vanilla` - Vanilla Minecraft
- `fabric` - Fabric loader
- `quilt` - Quilt loader
- `neoforge` - NeoForge loader
- `forge` - Forge loader (in progress)
- `forge_legacy` - Legacy Forge (in progress)
- `lighty_updater` - Custom loader system
- `all-loaders` - All of the above

**Feature-gated exports**:

```rust
// Only available with feature = "vanilla"
#[cfg(feature = "vanilla")]
use lighty_loaders::loaders::vanilla;

// Only available with feature = "fabric"
#[cfg(feature = "fabric")]
use lighty_loaders::loaders::fabric;
```

## Type Aliases

### Result Types

```rust
// In lighty_core
pub type Result<T> = std::result::Result<T, QueryError>;

// In lighty_loaders::utils::query
pub type Result<T> = std::result::Result<T, QueryError>;
```

Both are the same type. Use `anyhow::Result` for application code:

```rust
use anyhow::Result;

async fn my_function() -> Result<()> {
    let metadata = instance.get_metadata().await?;  // Works!
    Ok(())
}
```

## Version Metadata Types

All in `lighty_loaders::types::version_metadata`:

```rust
use lighty_loaders::types::version_metadata::{
    Version,          // Complete Minecraft version data
    VersionMetaData,  // Wrapper enum (Version or Version with mods)
    Library,          // Library dependency
    LibraryDownload,  // Library download info
    Arguments,        // JVM and game arguments
    AssetIndex,       // Asset index metadata
    JavaVersion,      // Java version requirement
    Downloads,        // Client/server download URLs
    // ... and more
};
```

Re-exported in `lighty_launcher`:

```rust
use lighty_launcher::loaders::version_metadata::{
    Version,
    VersionMetaData,
    // ...
};
```

## Summary

### Most Common Imports

```rust
// Recommended for applications
use lighty_launcher::prelude::*;

// For library development
use lighty_loaders::types::{Loader, VersionInfo, LoaderExtensions};
use lighty_core::QueryError;

// For implementing custom loaders
use lighty_loaders::utils::query::Query;
use lighty_loaders::utils::manifest::ManifestRepository;
```

### Trait Imports

```rust
// Always needed for using get_metadata(), get_libraries(), etc.
use lighty_launcher::loaders::LoaderExtensions;
// or
use lighty_launcher::prelude::*;  // includes LoaderExtensions
```

### Event Imports

```rust
// For event handling
use lighty_launcher::event::{Event, LoaderEvent, EventBus};
// or
use lighty_launcher::prelude::*;  // includes Event, LoaderEvent, EventBus
```

## Related Documentation

- [How to Use](./how-to-use.md) - Practical usage guide
- [Traits](./traits.md) - VersionInfo and LoaderExtensions
- [Events](./events.md) - LoaderEvent types
