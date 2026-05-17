# Traits

## VersionInfo

The `VersionInfo` trait provides a generic interface for representing version information. It's implemented by `VersionBuilder` and `LightyVersionBuilder`.

### Definition

```rust
pub trait VersionInfo: Clone + Send + Sync {
    type LoaderType: Clone + Send + Sync + std::fmt::Debug;

    fn name(&self) -> &str;
    fn loader_version(&self) -> &str;
    fn minecraft_version(&self) -> &str;
    fn game_dirs(&self) -> &Path;
    fn java_dirs(&self) -> &Path;
    fn loader(&self) -> &Self::LoaderType;

    // Helper methods
    fn game_dir_exists(&self) -> bool;
    fn java_dir_exists(&self) -> bool;
    fn full_identifier(&self) -> String;
    fn paths(&self) -> (&Path, &Path);
}
```

### Exports

**In lighty_loaders**:
```rust
use lighty_loaders::types::VersionInfo;
```

**In lighty_launcher**:
```rust
use lighty_launcher::loaders::VersionInfo;
// or
use lighty_launcher::prelude::VersionInfo;
```

### Methods

#### name() -> &str

Returns the instance name (unique identifier).

```rust
let instance = VersionBuilder::new("my-instance", Loader::Vanilla, "", "1.21.1");
println!("Instance name: {}", instance.name()); // "my-instance"
```

#### loader_version() -> &str

Returns the loader version string.

```rust
let instance = VersionBuilder::new("fabric-1.21", Loader::Fabric, "0.16.9", "1.21.1");
println!("Loader version: {}", instance.loader_version()); // "0.16.9"
```

For Vanilla, this returns an empty string.

#### minecraft_version() -> &str

Returns the Minecraft version.

```rust
println!("Minecraft: {}", instance.minecraft_version()); // "1.21.1"
```

#### game_dirs() -> &Path

Returns the game directory path (contains versions, libraries, assets).

```rust
let game_dir = instance.game_dirs();
// Typically: ~/.local/share/MyLauncher (Linux)
//            ~/Library/Application Support/MyLauncher (macOS)
//            C:\Users\{user}\AppData\Roaming\MyLauncher (Windows)
```

**Structure**:
```
game_dirs/
├── versions/          # Minecraft versions
│   └── {instance}/
├── libraries/         # Java libraries
├── assets/            # Game assets
└── instances/         # Instance data (mods, configs)
```

#### java_dirs() -> &Path

Returns the Java runtime directory path.

```rust
let java_dir = instance.java_dirs();
// Typically: ~/.cache/MyLauncher (Linux)
//            ~/Library/Caches/MyLauncher (macOS)
//            C:\Users\{user}\AppData\Local\MyLauncher\cache (Windows)
```

**Structure**:
```
java_dirs/
└── runtimes/          # Java installations
    ├── java-8/
    ├── java-17/
    └── java-21/
```

#### loader() -> &Loader

Returns the loader type.

```rust
match instance.loader() {
    Loader::Vanilla => println!("Pure Minecraft"),
    Loader::Fabric => println!("Fabric loader"),
    Loader::Quilt => println!("Quilt loader"),
    Loader::NeoForge => println!("NeoForge loader"),
    _ => {}
}
```

#### game_dir_exists() -> bool

Checks if the game directory exists on disk.

```rust
if instance.game_dir_exists() {
    println!("Game directory found");
} else {
    println!("Need to create game directory");
}
```

#### java_dir_exists() -> bool

Checks if the Java directory exists.

```rust
if instance.java_dir_exists() {
    println!("Java directory found");
}
```

#### full_identifier() -> String

Returns a complete identifier combining name, MC version, and loader version.

```rust
let id = instance.full_identifier();
// Format: "{name}-{minecraft_version}-{loader_version}"
// Example: "my-instance-1.21.1-0.16.9"
```

#### paths() -> (&Path, &Path)

Returns both game and Java directories as a tuple.

```rust
let (game_dir, java_dir) = instance.paths();
println!("Game: {}", game_dir.display());
println!("Java: {}", java_dir.display());
```

### Usage Example

```rust
use lighty_launcher::prelude::*;

fn print_version_info<V: VersionInfo>(version: &V) {
    println!("=== Version Info ===");
    println!("Name: {}", version.name());
    println!("Minecraft: {}", version.minecraft_version());
    println!("Loader: {:?}", version.loader());
    println!("Loader version: {}", version.loader_version());
    println!("Full ID: {}", version.full_identifier());
    println!("Game dir: {}", version.game_dirs().display());
    println!("Java dir: {}", version.java_dirs().display());
    println!("Game dir exists: {}", version.game_dir_exists());
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("MyLauncher")?;

    let instance = VersionBuilder::new("test", Loader::Fabric, "0.16.9", "1.21.1");

    print_version_info(&instance);

    Ok(())
}
```

## LoaderExtensions

The `LoaderExtensions` trait provides methods for fetching loader metadata. It's automatically implemented for any type that implements `VersionInfo<LoaderType = Loader>`.

### Definition

```rust
#[async_trait]
pub trait LoaderExtensions {
    async fn get_metadata(&self) -> Result<Arc<VersionMetaData>>;
    async fn get_libraries(&self) -> Result<Arc<VersionMetaData>>;
    async fn get_main_class(&self) -> Result<Arc<VersionMetaData>>;
    async fn get_natives(&self) -> Result<Arc<VersionMetaData>>;
    async fn get_java_version(&self) -> Result<Arc<VersionMetaData>>;
    async fn get_assets(&self) -> Result<Arc<VersionMetaData>>;
}
```

### Exports

**In lighty_loaders**:
```rust
use lighty_loaders::types::LoaderExtensions;
```

**In lighty_launcher**:
```rust
use lighty_launcher::loaders::LoaderExtensions;
// or
use lighty_launcher::prelude::LoaderExtensions;
```

### Methods

#### get_metadata() -> Result<Arc<VersionMetaData>>

Fetches complete metadata for the loader. This is the main method you should use.

```rust
use lighty_launcher::loaders::LoaderExtensions;

let metadata = instance.get_metadata().await?;

println!("Version: {}", metadata.id);
println!("Main class: {}", metadata.main_class);
println!("Libraries: {}", metadata.libraries.len());
```

**Returns**: `Arc<VersionMetaData>` containing:
- `id`: Minecraft version
- `main_class`: Entry point class
- `libraries`: Vec of library dependencies
- `arguments`: JVM and game arguments
- `asset_index`: Asset information
- And more...

**Loader dispatch**:
- `Vanilla` → Vanilla manifest
- `Fabric` → Vanilla + Fabric merged
- `Quilt` → Vanilla + Quilt merged
- `NeoForge` → NeoForge manifest
- `Forge` → Forge manifest (in progress)
- `LightyUpdater` → Custom server manifest

#### get_libraries() -> Result<Arc<VersionMetaData>>

Fetches only library metadata (faster than full metadata).

```rust
let libraries = instance.get_libraries().await?;

// Only libraries are guaranteed to be present
for lib in &libraries.libraries {
    println!("Library: {}", lib.name);
}
```

**Supported loaders**: Vanilla, Fabric, Quilt
**NeoForge**: Returns full metadata (no separate libraries query)

#### get_main_class() -> Result<Arc<VersionMetaData>>

Fetches only the main class information.

```rust
let main_class_data = instance.get_main_class().await?;
println!("Main class: {}", main_class_data.main_class);
```

**Requires**: `vanilla` feature
**Only for**: Vanilla-based queries

#### get_natives() -> Result<Arc<VersionMetaData>>

Fetches only native library information.

```rust
let natives = instance.get_natives().await?;

// Native libraries for platform-specific code
for lib in &natives.libraries {
    if lib.natives.is_some() {
        println!("Native: {}", lib.name);
    }
}
```

**Requires**: `vanilla` feature
**Only for**: Vanilla-based queries

#### get_java_version() -> Result<Arc<VersionMetaData>>

Fetches Java version requirement.

```rust
let java_info = instance.get_java_version().await?;

// Java version metadata
println!("Required Java: {}", java_info.java_version.major_version);
```

**Requires**: `vanilla` feature
**Only for**: Vanilla-based queries

#### get_assets() -> Result<Arc<VersionMetaData>>

Fetches asset information.

```rust
let assets = instance.get_assets().await?;

if let Some(asset_index) = &assets.asset_index {
    println!("Asset ID: {}", asset_index.id);
    println!("Total size: {} MB", asset_index.total_size / 1_000_000);
}
```

**Requires**: `vanilla` feature
**Only for**: Vanilla-based queries

### Usage Example

```rust
use lighty_launcher::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("MyLauncher")?;

    let instance = VersionBuilder::new("fabric-1.21", Loader::Fabric, "0.16.9", "1.21.1");

    // Full metadata (recommended)
    let metadata = instance.get_metadata().await?;
    println!("Full metadata: {} libraries", metadata.libraries.len());

    // Specific queries (optional, for optimization)
    let libraries = instance.get_libraries().await?;
    println!("Libraries only: {}", libraries.libraries.len());

    // Vanilla-specific queries
    if matches!(instance.loader(), Loader::Vanilla) {
        let assets = instance.get_assets().await?;
        if let Some(idx) = &assets.asset_index {
            println!("Assets: {} ({})", idx.id, idx.url);
        }
    }

    Ok(())
}
```

### Error Handling

```rust
use lighty_core::QueryError;

match instance.get_metadata().await {
    Ok(metadata) => {
        println!("Success!");
    }
    Err(QueryError::UnsupportedLoader(msg)) => {
        eprintln!("Loader not supported: {}", msg);
        // Feature might not be enabled
    }
    Err(QueryError::NetworkError(e)) => {
        eprintln!("Network error: {}", e);
    }
    Err(e) => {
        eprintln!("Error: {:?}", e);
    }
}
```

## Custom Implementations

You can implement these traits for your own types:

```rust
use lighty_loaders::types::{VersionInfo, Loader};
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct MyCustomVersion {
    name: String,
    mc_version: String,
    game_path: PathBuf,
    java_path: PathBuf,
}

impl VersionInfo for MyCustomVersion {
    type LoaderType = Loader;

    fn name(&self) -> &str {
        &self.name
    }

    fn loader_version(&self) -> &str {
        ""  // Custom implementation
    }

    fn minecraft_version(&self) -> &str {
        &self.mc_version
    }

    fn game_dirs(&self) -> &Path {
        &self.game_path
    }

    fn java_dirs(&self) -> &Path {
        &self.java_path
    }

    fn loader(&self) -> &Loader {
        &Loader::Vanilla  // Or your custom loader
    }
}

// LoaderExtensions is automatically implemented!
```

Now `MyCustomVersion` can use all `LoaderExtensions` methods:

```rust
let custom = MyCustomVersion { /* ... */ };
let metadata = custom.get_metadata().await?;  // Works!
```

## Related Documentation

- [How to Use](./how-to-use.md) - Practical usage examples
- [Query System](./query.md) - Understanding queries
- [Cache System](./cache.md) - How caching works
- [Exports](./exports.md) - All exports and paths
