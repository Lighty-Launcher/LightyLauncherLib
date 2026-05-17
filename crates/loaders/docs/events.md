# LoaderEvent Types

Events emitted by `lighty-loaders` during metadata fetching and processing.

## Event Types

All events are part of the `LoaderEvent` enum:

```rust
use lighty_launcher::event::LoaderEvent;
// or
use lighty_event::LoaderEvent;
```

### FetchingData

Emitted when starting to fetch loader manifest from the API.

**Fields**:
- `loader`: String - Loader name (e.g., "Vanilla", "Fabric", "Quilt")
- `minecraft_version`: String - Minecraft version (e.g., "1.21.1")
- `loader_version`: String - Loader version (e.g., "0.16.9" for Fabric, "" for Vanilla)

**When**:
- Before HTTP request to loader API
- Before checking cache

**Example**:
```rust
Event::Loader(LoaderEvent::FetchingData {
    loader,
    minecraft_version,
    loader_version
}) => {
    trace_info!("Fetching {} for MC {} (loader: {})",
        loader, minecraft_version, loader_version);
}
```

### DataFetched

Emitted when loader manifest has been successfully retrieved.

**Fields**:
- `loader`: String - Loader name
- `minecraft_version`: String - Minecraft version
- `loader_version`: String - Loader version

**When**:
- After successful API response
- After parsing JSON

**Example**:
```rust
Event::Loader(LoaderEvent::DataFetched {
    loader,
    minecraft_version,
    loader_version
}) => {
    trace_info!("{} data fetched for MC {} ({})",
        loader, minecraft_version, loader_version);
}
```

### ManifestNotFound

Emitted when the requested version is not found (404 or similar).

**Fields**:
- `loader`: String - Loader name
- `minecraft_version`: String - Minecraft version
- `loader_version`: String - Loader version
- `error`: String - Error description

**When**:
- Version doesn't exist
- API returns 404
- Invalid version format

**Example**:
```rust
Event::Loader(LoaderEvent::ManifestNotFound {
    loader,
    minecraft_version,
    loader_version,
    error
}) => {
    trace_error!("{} {} not found for MC {}: {}",
        loader, loader_version, minecraft_version, error);
}
```

### ManifestCached

Emitted when using a cached manifest instead of fetching from API.

**Fields**:
- `loader`: String - Loader name

**When**:
- Cache hit (TTL not expired)
- Metadata already in cache

**Example**:
```rust
Event::Loader(LoaderEvent::ManifestCached { loader }) => {
    trace_info!("Using cached {} manifest", loader);
}
```

### MergingLoaderData

Emitted when merging two loader metadata (e.g., Fabric + Vanilla).

**Fields**:
- `base_loader`: String - Base loader (e.g., "Vanilla")
- `overlay_loader`: String - Overlay loader (e.g., "Fabric")

**When**:
- Combining Fabric/Quilt with Vanilla
- Merging loader libraries with base game

**Example**:
```rust
Event::Loader(LoaderEvent::MergingLoaderData {
    base_loader,
    overlay_loader
}) => {
    trace_info!("Merging {} with {}", overlay_loader, base_loader);
}
```

### DataMerged

Emitted when loader data merge is complete.

**Fields**:
- `base_loader`: String - Base loader
- `overlay_loader`: String - Overlay loader

**When**:
- After successful merge
- Combined metadata ready

**Example**:
```rust
Event::Loader(LoaderEvent::DataMerged {
    base_loader,
    overlay_loader
}) => {
    trace_info!("{} and {} merged successfully", overlay_loader, base_loader);
}
```

## Complete Example

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
            if let Event::Loader(loader_event) = event {
                match loader_event {
                    LoaderEvent::FetchingData { loader, minecraft_version, loader_version } => {
                        println!("🔄 Fetching {} for MC {} (version: {})",
                            loader, minecraft_version, loader_version);
                    }
                    LoaderEvent::DataFetched { loader, .. } => {
                        println!("✅ {} data fetched", loader);
                    }
                    LoaderEvent::ManifestNotFound { error, .. } => {
                        println!("❌ Not found: {}", error);
                    }
                    LoaderEvent::ManifestCached { loader } => {
                        println!("💾 Using cached {} manifest", loader);
                    }
                    LoaderEvent::MergingLoaderData { base_loader, overlay_loader } => {
                        println!("🔀 Merging {} + {}", overlay_loader, base_loader);
                    }
                    LoaderEvent::DataMerged { overlay_loader, .. } => {
                        println!("✅ {} merge complete", overlay_loader);
                    }
                }
            }
        }
    });

    // Create instance (will emit events)
    let instance = VersionBuilder::new("fabric-1.21", Loader::Fabric, "0.16.9", "1.21.1");

    // This will trigger LoaderEvent emissions
    let _metadata = instance.get_metadata().await?;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    Ok(())
}
```

## Typical Event Sequences

### Vanilla (First Time)

```
FetchingData { loader: "Vanilla", ... }
DataFetched { loader: "Vanilla", ... }
```

### Vanilla (Cached)

```
ManifestCached { loader: "Vanilla" }
```

### Fabric (First Time)

```
FetchingData { loader: "Vanilla", ... }
DataFetched { loader: "Vanilla", ... }
FetchingData { loader: "Fabric", ... }
DataFetched { loader: "Fabric", ... }
MergingLoaderData { base_loader: "Vanilla", overlay_loader: "Fabric" }
DataMerged { base_loader: "Vanilla", overlay_loader: "Fabric" }
```

### Fabric (Partially Cached)

```
ManifestCached { loader: "Vanilla" }
FetchingData { loader: "Fabric", ... }
DataFetched { loader: "Fabric", ... }
MergingLoaderData { base_loader: "Vanilla", overlay_loader: "Fabric" }
DataMerged { base_loader: "Vanilla", overlay_loader: "Fabric" }
```

### Not Found

```
FetchingData { loader: "Vanilla", ... }
ManifestNotFound { loader: "Vanilla", error: "Version 1.99.99 not found", ... }
```

## Exports

**In lighty_event**:
```rust
use lighty_event::{Event, LoaderEvent};
```

**In lighty_launcher**:
```rust
use lighty_launcher::event::{Event, LoaderEvent};
// or
use lighty_launcher::prelude::*;
```

## Related Documentation

- [Event System](../../event/README.md) - EventBus and EventReceiver
- [How to Use](./how-to-use.md) - Using events with loaders
- [Cache System](./cache.md) - Understanding cache hits
