# How to Use lighty-loaders

## Basic Usage

### Step 1: Initialize AppState

```rust
use lighty_launcher::core::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("MyLauncher")?;

    // AppState exposes:
    // - data_dir()  -> game files, instances, versions
    // - cache_dir() -> java runtimes, temporary files

    Ok(())
}
```

### Step 2: Create a VersionBuilder

```rust
use lighty_launcher::prelude::*;

let instance = VersionBuilder::new(
    "my-instance",      // Instance name
    Loader::Vanilla,    // Loader type
    "",                 // Loader version (empty for Vanilla)
    "1.21.1",           // Minecraft version
);
```

### Step 3: Get Metadata

```rust
use lighty_launcher::loaders::LoaderExtensions;

// Fetch metadata (automatically cached)
let metadata = instance.get_metadata().await?;

// Access metadata
println!("Version: {}", metadata.id);
println!("Main class: {}", metadata.main_class);
println!("Libraries: {}", metadata.libraries.len());
```

## Loaders Examples

### Vanilla Minecraft

```rust
use lighty_launcher::prelude::*;

let instance = VersionBuilder::new("vanilla-1.21", Loader::Vanilla, "", "1.21.1");
let metadata = instance.get_metadata().await?;
```

**Exports**:
- Crate: `lighty_loaders::loaders::vanilla`
- Main: `lighty_launcher::loaders::vanilla`

### Fabric

```rust
let instance = VersionBuilder::new(
    "fabric-1.21",
    Loader::Fabric,
    "0.16.9",      // Fabric loader version
    "1.21.1",
);
```

**Exports**:
- Crate: `lighty_loaders::loaders::fabric`
- Main: `lighty_launcher::loaders::fabric`

### Quilt

```rust
let instance = VersionBuilder::new(
    "quilt-1.21",
    Loader::Quilt,
    "0.27.1",      // Quilt loader version
    "1.21.1",
);
```

**Exports**:
- Crate: `lighty_loaders::loaders::quilt`
- Main: `lighty_launcher::loaders::quilt`

### NeoForge

```rust
let instance = VersionBuilder::new(
    "neoforge-1.21",
    Loader::NeoForge,
    "21.1.80",     // NeoForge version
    "1.21.1",
);
```

**Exports**:
- Crate: `lighty_loaders::loaders::neoforge`
- Main: `lighty_launcher::loaders::neoforge`

### LightyUpdater with LightyVersionBuilder

For custom servers with LightyUpdater:

```rust
use lighty_launcher::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("MyLauncher")?;

    // LightyVersionBuilder for custom servers
    let instance = LightyVersionBuilder::new(
        "my-modpack",                    // Instance name
        "https://myserver.com/api",      // Server URL
    );

    let metadata = instance.get_metadata().await?;

    println!("Custom modpack: {}", metadata.id);

    Ok(())
}
```

**Exports**:
- Builder: `lighty_version::LightyVersionBuilder`
- Re-export: `lighty_launcher::version::LightyVersionBuilder`
- Loader module: `lighty_loaders::loaders::lighty_updater`
- Re-export: `lighty_launcher::loaders::lighty_updater`

**For more details**, see:
- [LightyUpdater Documentation](./loaders/lighty_updater.md)
- [LightyUpdater GitHub Repository](https://github.com/Lighty-Launcher/LightyUpdater)

## Advanced Usage

### Query Specific Metadata

Instead of fetching full metadata, you can query specific parts:

```rust
use lighty_launcher::loaders::LoaderExtensions;

// Get only libraries
let libraries = instance.get_libraries().await?;

// Get only assets (Vanilla-based only)
let assets = instance.get_assets().await?;

// Get only main class (Vanilla-based only)
let main_class = instance.get_main_class().await?;

// Get Java version requirement (Vanilla-based only)
let java_ver = instance.get_java_version().await?;

// Get native libraries (Vanilla-based only)
let natives = instance.get_natives().await?;
```

### With Events

```rust
use lighty_launcher::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("MyLauncher")?;

    // Create event bus
    let event_bus = EventBus::new(1000);
    let mut receiver = event_bus.subscribe();

    // Spawn listener
    tokio::spawn(async move {
        while let Ok(event) = receiver.next().await {
            match event {
                Event::Loader(LoaderEvent::FetchingData { loader, minecraft_version, .. }) => {
                    trace_info!("Fetching {} for MC {}", loader, minecraft_version);
                }
                Event::Loader(LoaderEvent::DataFetched { loader, .. }) => {
                    trace_info!("{} data fetched!", loader);
                }
                Event::Loader(LoaderEvent::ManifestCached { loader }) => {
                    trace_info!("Using cached {} manifest", loader);
                }
                _ => {}
            }
        }
    });

    let instance = VersionBuilder::new("fabric-1.21", Loader::Fabric, "0.16.9", "1.21.1");

    // Metadata fetching will emit events
    let metadata = instance.get_metadata().await?;

    Ok(())
}
```

**Exports**:
- Event types: `lighty_event::LoaderEvent`
- Re-export: `lighty_launcher::event::LoaderEvent`

See [Events](./events.md) for all event types.

### Instance Size Calculation

```rust
use lighty_launcher::loaders::InstanceSize;
use lighty_launcher::launch::InstanceControl;

let metadata = instance.get_metadata().await?;

// Extract Version from Arc<VersionMetaData>
use lighty_launcher::loaders::VersionMetaData;
let version = match metadata.as_ref() {
    VersionMetaData::Version(v) => v,
    _ => panic!("Expected Version metadata"),
};

// Calculate size
let size = instance.size_of_instance(version);

println!("Libraries: {}", InstanceSize::format(size.libraries));
println!("Client: {}", InstanceSize::format(size.client));
println!("Assets: {}", InstanceSize::format(size.assets));
println!("Mods: {}", InstanceSize::format(size.mods));
println!("Natives: {}", InstanceSize::format(size.natives));
println!("Total: {} ({:.2} GB)", InstanceSize::format(size.total), size.total_gb());
```

**Exports**:
- Type: `lighty_loaders::types::InstanceSize`
- Re-export: `lighty_launcher::loaders::InstanceSize`
- Trait: `lighty_launch::InstanceControl::size_of_instance()`
- Re-export: `lighty_launcher::launch::InstanceControl::size_of_instance()`

## Feature Flags

Enable only the loaders you need:

```toml
[dependencies]
# All loaders
lighty-launcher = { version = "26.5.1", features = ["all-loaders"] }

# Specific loaders
lighty-launcher = { version = "26.5.1", features = ["vanilla", "fabric", "quilt", "neoforge"] }
```

Available features:
- `vanilla` - Vanilla Minecraft
- `fabric` - Fabric loader
- `quilt` - Quilt loader
- `neoforge` - NeoForge loader
- `forge` - Forge loader (1.13+, in progress)
- `forge_legacy` - Forge Legacy (1.7-1.12, in progress)
- `lighty_updater` - Custom loader system
- `all-loaders` - All of the above

## Error Handling

```rust
use lighty_core::QueryError;

match instance.get_metadata().await {
    Ok(metadata) => {
        println!("Success! Libraries: {}", metadata.libraries.len());
    }
    Err(QueryError::NetworkError(e)) => {
        eprintln!("Network error: {}", e);
    }
    Err(QueryError::NotFound(v)) => {
        eprintln!("Version not found: {}", v);
    }
    Err(QueryError::ParseError(e)) => {
        eprintln!("Parse error: {}", e);
    }
    Err(QueryError::UnsupportedLoader(l)) => {
        eprintln!("Unsupported loader: {}", l);
    }
    Err(e) => {
        eprintln!("Error: {:?}", e);
    }
}
```

**Exports**:
- Type: `lighty_core::QueryError`
- Not re-exported in main crate (use full path)

## Related Documentation

- [Traits](./traits.md) - Understanding VersionInfo and LoaderExtensions
- [Query System](./query.md) - How the query system works
- [Cache System](./cache.md) - Caching architecture
- [Events](./events.md) - All LoaderEvent types
- [Exports](./exports.md) - Complete export reference

## Related Crates

- **[lighty-version](../../version/README.md)** - Implements VersionBuilder
- **[lighty-launch](../../launch/README.md)** - For launching instances
- **[lighty-event](../../event/README.md)** - Event system
