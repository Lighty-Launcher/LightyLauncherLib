# Arguments System

## Overview

The arguments system builds the complete command-line arguments for launching Minecraft. It handles JVM arguments, game arguments, and variable substitution.

## Placeholders (Variable Substitution)

The launch system uses placeholders that are replaced with actual values at launch time.

### Authentication Placeholders

| Placeholder | Description | Example Value |
|-------------|-------------|---------------|
| `${auth_player_name}` | Player username | `"Player123"` |
| `${auth_uuid}` | Player UUID | `"550e8400-e29b-41d4-a716-446655440000"` |
| `${auth_access_token}` | Access token | `"eyJhbGc..."` or `"0"` (offline) |
| `${auth_xuid}` | Xbox User ID | `"2535405290..."` or `"0"` |
| `${clientid}` | Client ID | `"{client-id}"` |
| `${user_type}` | User type | `"legacy"` or `"msa"` |
| `${user_properties}` | User properties JSON | `"{}"` |

### Directory Placeholders

| Placeholder | Description | Example Path |
|-------------|-------------|--------------|
| `${game_directory}` | Game instance directory | `/home/user/.local/share/MyLauncher/instance` |
| `${assets_root}` | Assets root directory | `/home/user/.local/share/MyLauncher/assets` |
| `${natives_directory}` | Native libraries directory | `/tmp/natives-xxxxx` |
| `${library_directory}` | Libraries directory | `/home/user/.local/share/MyLauncher/libraries` |
| `${classpath}` | Java classpath | `/path/lib1.jar:/path/lib2.jar:...` |
| `${classpath_separator}` | Platform separator | `:` (Linux/macOS) or `;` (Windows) |

### Version Placeholders

| Placeholder | Description | Example Value |
|-------------|-------------|---------------|
| `${version_name}` | Minecraft version | `"1.21.1"` |
| `${version_type}` | Version type | `"release"` or `"snapshot"` |
| `${assets_index_name}` | Asset index ID | `"16"` |
| `${launcher_name}` | Launcher name | `"MyLauncher"` |
| `${launcher_version}` | Launcher version | `"26.5.1"` |

## JVM Arguments

### Default JVM Arguments

When no JVM arguments are specified in version metadata, these defaults are used:

```
-Djava.library.path=${natives_directory}
-Dminecraft.launcher.brand=${launcher_name}
-Dminecraft.launcher.version=${launcher_version}
-Xmx2G
-XX:+UnlockExperimentalVMOptions
-XX:+UseG1GC
-XX:G1NewSizePercent=20
-XX:G1ReservePercent=20
-XX:MaxGCPauseMillis=50
-XX:G1HeapRegionSize=32M
-cp ${classpath}
```

### Custom JVM Arguments

You can customize JVM arguments using the `with_jvm_options()` builder:

```rust
use lighty_launch::InstanceControl;

instance.launch(&profile, JavaDistribution::Temurin)
    .with_jvm_options()
        .set("Xmx", "4G")                          // Maximum heap
        .set("Xms", "2G")                          // Initial heap
        .set("XX:+UseG1GC", "")                    // G1 garbage collector
        .set("XX:MaxGCPauseMillis", "50")          // Max GC pause
        .set("Dfile.encoding", "UTF-8")            // File encoding
        .done()
    .run()
    .await?;
```

**Common JVM Options**:

| Option | Description | Example |
|--------|-------------|---------|
| `-Xmx` | Maximum heap size | `"4G"`, `"8G"` |
| `-Xms` | Initial heap size | `"2G"`, `"4G"` |
| `-XX:+UseG1GC` | Use G1 garbage collector | `""` |
| `-XX:+UseZGC` | Use Z garbage collector (Java 15+) | `""` |
| `-XX:MaxGCPauseMillis` | Target max GC pause time (ms) | `"50"`, `"100"` |
| `-XX:G1HeapRegionSize` | G1 heap region size | `"32M"`, `"16M"` |
| `-Dfile.encoding` | File encoding | `"UTF-8"` |
| `-Djava.net.preferIPv4Stack` | Prefer IPv4 | `"true"` |

### Critical JVM Arguments (Always Present)

These arguments are automatically added if not present:

1. **`-Djava.library.path=${natives_directory}`**
   - Required for LWJGL native libraries
   - Automatically set to temporary natives directory

2. **`-Dminecraft.launcher.brand=${launcher_name}`**
   - Identifies the launcher
   - Set from AppState organization name

3. **`-Dminecraft.launcher.version=${launcher_version}`**
   - Launcher version
   - Set from package version

4. **`-cp ${classpath}`**
   - Java classpath with all libraries
   - Must be the last JVM argument before main class

## Game Arguments

### Standard Game Arguments

Game arguments are provided by the version metadata:

```
--username ${auth_player_name}
--version ${version_name}
--gameDir ${game_directory}
--assetsDir ${assets_root}
--assetIndex ${assets_index_name}
--uuid ${auth_uuid}
--accessToken ${auth_access_token}
--clientId ${clientid}
--xuid ${auth_xuid}
--userType ${user_type}
--versionType ${version_type}
```

### Custom Game Arguments

You can add or override game arguments:

```rust
instance.launch(&profile, JavaDistribution::Temurin)
    .with_game_options()
        .set("width", "1920")
        .set("height", "1080")
        .set("fullscreen", "true")
        .set("quickPlayPath", "servers.dat")
        .done()
    .run()
    .await?;
```

**Common Game Options**:

| Option | Description | Example |
|--------|-------------|---------|
| `--width` | Window width | `"1920"`, `"2560"` |
| `--height` | Window height | `"1080"`, `"1440"` |
| `--fullscreen` | Fullscreen mode | `"true"`, `"false"` |
| `--quickPlayPath` | Quick play server file | `"servers.dat"` |
| `--quickPlaySingleplayer` | Quick play world | `"New World"` |
| `--quickPlayMultiplayer` | Quick play server | `"mc.hypixel.net"` |
| `--quickPlayRealms` | Quick play realm | `"realm-name"` |
| `--demo` | Demo mode | `"true"` |
| `--server` | Auto-connect server | `"play.example.com"` |
| `--port` | Server port | `"25565"` |

## Complete Argument Flow

### 1. Variable Map Creation

All placeholders are populated with actual values:

```rust
let mut variables = HashMap::new();
variables.insert("auth_player_name", "Player123");
variables.insert("auth_uuid", "550e8400-...");
variables.insert("game_directory", "/home/user/.local/share/MyLauncher/instance");
// ... etc
```

### 2. Argument Overrides Applied

Custom options override default variables:

```rust
// User sets custom resolution
with_game_options()
    .set("width", "1920")
    .set("height", "1080")

// Overrides applied to variables map
variables.insert("width", "1920");
variables.insert("height", "1080");
```

### 3. Variable Substitution

Placeholders in arguments are replaced:

```
Before: --username ${auth_player_name}
After:  --username Player123

Before: --gameDir ${game_directory}
After:  --gameDir /home/user/.local/share/MyLauncher/instance
```

### 4. JVM Arguments Processing

```rust
// 1. Start with version metadata JVM args (or defaults)
let mut jvm_args = builder.arguments.jvm;

// 2. Apply custom JVM options
with_jvm_options().set("Xmx", "4G") // → -Xmx4G

// 3. Ensure critical args are present
if !has("-Djava.library.path=") {
    jvm_args.insert(0, "-Djava.library.path=/tmp/natives-xxxxx");
}

// 4. Add classpath (must be last)
jvm_args.push("-cp");
jvm_args.push("/path/lib1.jar:/path/lib2.jar:...");
```

### 5. Final Command Assembly

```
[JVM args] + [Main class] + [Game args] + [Raw args]
```

**Example**:
```bash
java \
  -Djava.library.path=/tmp/natives-xxxxx \
  -Dminecraft.launcher.brand=MyLauncher \
  -Dminecraft.launcher.version=26.5.1 \
  -Xmx4G \
  -Xms2G \
  -XX:+UseG1GC \
  -cp /path/lib1.jar:/path/lib2.jar:... \
  net.minecraft.client.main.Main \
  --username Player123 \
  --version 1.21.1 \
  --gameDir /home/user/.local/share/MyLauncher/instance \
  --assetsDir /home/user/.local/share/MyLauncher/assets \
  --assetIndex 16 \
  --uuid 550e8400-e29b-41d4-a716-446655440000 \
  --accessToken 0 \
  --width 1920 \
  --height 1080
```

## Argument Removal

You can remove specific arguments:

```rust
instance.launch(&profile, JavaDistribution::Temurin)
    .with_jvm_options()
        .remove("XX:+UnlockExperimentalVMOptions")  // Remove this JVM arg
        .done()
    .with_game_options()
        .remove("demo")  // Remove demo mode arg
        .done()
    .run()
    .await?;
```

## Platform-Specific Arguments

### Classpath Separator

Automatically set based on platform:
- **Windows**: `;` (semicolon)
- **Linux/macOS**: `:` (colon)

### Natives Directory

Platform-specific native libraries:
- **Windows**: `lwjgl-3.3.3-natives-windows.jar`
- **Linux**: `lwjgl-3.3.3-natives-linux.jar`
- **macOS**: `lwjgl-3.3.3-natives-macos.jar`

## Argument Constants

Available in `lighty_launch::arguments`:

```rust
use lighty_launch::arguments::{
    // Authentication
    KEY_AUTH_PLAYER_NAME,
    KEY_AUTH_UUID,
    KEY_AUTH_ACCESS_TOKEN,
    KEY_AUTH_XUID,
    KEY_CLIENT_ID,
    KEY_USER_TYPE,
    KEY_USER_PROPERTIES,

    // Version
    KEY_VERSION_NAME,
    KEY_VERSION_TYPE,

    // Directories
    KEY_GAME_DIRECTORY,
    KEY_ASSETS_ROOT,
    KEY_NATIVES_DIRECTORY,
    KEY_LIBRARY_DIRECTORY,
    KEY_ASSETS_INDEX_NAME,

    // Launcher
    KEY_LAUNCHER_NAME,
    KEY_LAUNCHER_VERSION,

    // Classpath
    KEY_CLASSPATH,
    KEY_CLASSPATH_SEPARATOR,
};
```

### Constant to Placeholder Mapping

| Constant | Placeholder | Value |
|----------|-------------|-------|
| `KEY_AUTH_PLAYER_NAME` | `${auth_player_name}` | `"auth_player_name"` |
| `KEY_AUTH_UUID` | `${auth_uuid}` | `"auth_uuid"` |
| `KEY_AUTH_ACCESS_TOKEN` | `${auth_access_token}` | `"auth_access_token"` |
| `KEY_AUTH_XUID` | `${auth_xuid}` | `"auth_xuid"` |
| `KEY_CLIENT_ID` | `${clientid}` | `"clientid"` |
| `KEY_USER_TYPE` | `${user_type}` | `"user_type"` |
| `KEY_USER_PROPERTIES` | `${user_properties}` | `"user_properties"` |
| `KEY_VERSION_NAME` | `${version_name}` | `"version_name"` |
| `KEY_VERSION_TYPE` | `${version_type}` | `"version_type"` |
| `KEY_GAME_DIRECTORY` | `${game_directory}` | `"game_directory"` |
| `KEY_ASSETS_ROOT` | `${assets_root}` | `"assets_root"` |
| `KEY_NATIVES_DIRECTORY` | `${natives_directory}` | `"natives_directory"` |
| `KEY_LIBRARY_DIRECTORY` | `${library_directory}` | `"library_directory"` |
| `KEY_ASSETS_INDEX_NAME` | `${assets_index_name}` | `"assets_index_name"` |
| `KEY_LAUNCHER_NAME` | `${launcher_name}` | `"launcher_name"` |
| `KEY_LAUNCHER_VERSION` | `${launcher_version}` | `"launcher_version"` |
| `KEY_CLASSPATH` | `${classpath}` | `"classpath"` |
| `KEY_CLASSPATH_SEPARATOR` | `${classpath_separator}` | `"classpath_separator"` |

**Usage example**:

```rust
use lighty_launch::arguments::{KEY_LAUNCHER_NAME, KEY_WIDTH, KEY_HEIGHT};

instance.launch(&profile, JavaDistribution::Temurin)
    .with_arguments()
        .set(KEY_LAUNCHER_NAME, "MyCustomLauncher")  // Override ${launcher_name}
        .set("width", "1920")                        // Adds --width 1920
        .set("height", "1080")                       // Adds --height 1080
        .done()
    .run()
    .await?;
```

## Complete Example

```rust
use lighty_core::AppState;
use lighty_launcher::prelude::*;
use lighty_java::JavaDistribution;
use lighty_launch::InstanceControl;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("MyLauncher")?;

    let mut instance = VersionBuilder::new(
        "optimized-1.21",
        Loader::Fabric,
        "0.16.9",
        "1.21.1",
    );

    let mut auth = OfflineAuth::new("Player123");

    #[cfg(not(feature = "events"))]
    let profile = auth.authenticate().await?;

    // Launch with custom arguments
    instance.launch(&profile, JavaDistribution::Temurin)
        .with_jvm_options()
            // Memory settings
            .set("Xmx", "6G")
            .set("Xms", "2G")

            // Garbage collection
            .set("XX:+UseG1GC", "")
            .set("XX:MaxGCPauseMillis", "50")
            .set("XX:G1HeapRegionSize", "32M")

            // Performance
            .set("XX:+UnlockExperimentalVMOptions", "")
            .set("XX:+AlwaysPreTouch", "")

            // System properties
            .set("Dfile.encoding", "UTF-8")
            .set("Djava.net.preferIPv4Stack", "true")
            .done()
        .with_game_options()
            // Window settings
            .set("width", "1920")
            .set("height", "1080")
            .set("fullscreen", "false")
            .done()
        .run()
        .await?;

    println!("Game launched!");

    Ok(())
}
```

## Related Documentation

- [How to Use](./how-to-use.md) - Practical examples
- [Launch Process](./launch.md) - Complete launch workflow
- [Instance Control](./instance-control.md) - Process management
