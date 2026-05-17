# lighty-java

Automatic Java Runtime Environment (JRE) management for Minecraft launchers with multi-distribution support.

## Overview

`lighty-java` provides automated downloading, installation, and management of Java runtimes for Minecraft:
- **Automatic JRE Download** - Download Java on-demand based on Minecraft version requirements
- **Multi-Distribution Support** - Temurin, GraalVM, Zulu, and Liberica distributions
- **Cross-Platform** - Windows, Linux, and macOS (x64 and ARM64)
- **Version Detection** - Automatically detect required Java version for any Minecraft version
- **Progress Tracking** - Real-time download and extraction progress via event system

## Quick Start

```toml
[dependencies]
lighty-java = "26.5.1"
```

### Basic Usage

```rust
use lighty_java::{JavaDistribution, jre_downloader};
use std::path::Path;

#[tokio::main]
async fn main() {
    let runtime_dir = Path::new("./runtimes");

    // Download Java 21 (Temurin distribution)
    let java_path = jre_downloader::jre_download(
        runtime_dir,
        &JavaDistribution::Temurin,
        &21,
        |current, total| {
            let percent = (current * 100) / total;
            println!("Download progress: {}%", percent);
        }
    ).await.unwrap();

    println!("Java installed at: {}", java_path.display());
}
```

### Version Detection

```rust
use lighty_java::runtime::get_jre_version;

// Minecraft 1.20.4 requires Java 17
let required_version = get_jre_version("1.20.4");
println!("Minecraft 1.20.4 requires Java {}", required_version);  // 17

// Minecraft 1.16.5 requires Java 8
let required_version = get_jre_version("1.16.5");
println!("Minecraft 1.16.5 requires Java {}", required_version);  // 8
```

## Java Distributions

| Distribution | Provider | Supported Versions | Best For |
|--------------|----------|-------------------|----------|
| **Temurin** (Recommended) | Eclipse Adoptium | 8, 11, 17, 21 | General use, maximum compatibility |
| **GraalVM** | Oracle | 17, 21 | Modern Minecraft (1.17+), maximum performance |
| **Zulu** | Azul Systems | 8, 11, 17, 21 | Enterprise environments, certified deployments |
| **Liberica** | BellSoft | 8, 11, 17, 21 | Resource-constrained systems, lightweight deployments |

## Platform Support

| Platform | Architecture | Temurin | GraalVM | Zulu | Liberica |
|----------|-------------|---------|---------|------|----------|
| Windows | x64 | ✅ | ✅ | ✅ | ✅ |
| Windows | ARM64 | ✅ | ❌ | ✅ | ✅ |
| Linux | x64 | ✅ | ✅ | ✅ | ✅ |
| Linux | ARM64 | ✅ | ✅ | ✅ | ✅ |
| macOS | x64 | ✅ | ✅ | ✅ | ✅ |
| macOS | ARM64 (M1/M2) | ✅ | ✅ | ✅ | ✅ |

## Complete Example

```rust
use lighty_java::{JavaDistribution, jre_downloader, runtime};
use std::path::Path;

#[tokio::main]
async fn main() {
    let runtime_dir = Path::new("./runtimes");
    let minecraft_version = "1.20.4";

    // 1. Detect required Java version
    let java_version = runtime::get_jre_version(minecraft_version);
    println!("Minecraft {} requires Java {}", minecraft_version, java_version);

    // 2. Check if Java is already installed
    let java_path = match jre_downloader::find_java_binary(
        runtime_dir,
        &JavaDistribution::Temurin,
        &java_version
    ).await {
        Ok(path) => {
            println!("Using existing Java at: {}", path.display());
            path
        }
        Err(_) => {
            // 3. Download and install Java
            println!("Downloading Java {}...", java_version);
            jre_downloader::jre_download(
                runtime_dir,
                &JavaDistribution::Temurin,
                &java_version,
                |current, total| {
                    let percent = (current * 100) / total;
                    print!("\rProgress: {}%", percent);
                }
            ).await.unwrap()
        }
    };

    // 4. Run Java process
    let mut java_runtime = runtime::JavaRuntime::new(&java_path);
    java_runtime.add_arg("-version");

    java_runtime.run(
        |line| println!("[OUT] {}", line),
        |line| eprintln!("[ERR] {}", line),
    ).await.unwrap();
}
```

## Documentation

📚 **[Complete Documentation](./docs)**

| Guide | Description |
|-------|-------------|
| [Overview](./docs/overview.md) | Design and implementation details |
| [Distributions](./docs/distributions.md) | Deep dive into each Java distribution |
| [Installation](./docs/installation.md) | Download and installation process |
| [Runtime Execution](./docs/runtime.md) | Java process execution and I/O handling |
| [Examples](./docs/examples.md) | Complete usage examples and patterns |

## License

MIT

## Links

- **Main Package**: [lighty-launcher](https://crates.io/crates/lighty-launcher)
- **Repository**: [GitHub](https://github.com/Lighty-Launcher/LightyLauncherLib)
- **Documentation**: [docs.rs/lighty-java](https://docs.rs/lighty-java)
