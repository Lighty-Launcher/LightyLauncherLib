# LightyLauncher

[![Crates.io](https://img.shields.io/crates/v/lighty-launcher.svg)](https://crates.io/crates/lighty-launcher)
[![Documentation](https://img.shields.io/badge/docs-gitbook-blue.svg)](https://hamadi.gitbook.io/lightylauncher)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rust-1.95%2B-red.svg)](https://www.rust-lang.org)

<p align="center">
  <img src="docs/assets/banner.png" alt="LightyLauncher banner" />
</p>

## A launcher built with LightyLauncherLib

<p align="center">
  <video src="https://github.com/Lighty-Launcher/LightyLauncherLib/releases/download/video-demo/exemple_launcher_with_lightylauncherlib.mp4" controls width="100%">
    Your browser doesn't render inline video —
    <a href="https://github.com/Lighty-Launcher/LightyLauncherLib/releases/download/video-demo/exemple_launcher_with_lightylauncherlib.mp4">click to watch the demo</a>.
  </video>
</p>

---

## Table of Contents

- [Why LightyLauncher](#why-lightylauncher)
- [Quick Start](#quick-start)
- [Architecture](#architecture)
- [Documentation](#documentation)
- [Cargo Features](#cargo-features)
- [Contributing](#contributing)
- [License](#license)
- [Related Projects](#related-projects)

---

## Why LightyLauncherLib

First off, it's:

- **Light.** Only ships what you use. A pure-Fabric build never pulls in the Forge pipeline.
- **Fast.** Mods, libs, assets, resourcepacks, shaders — all downloaded in parallel, not one after another.
- **Mods & modpacks just work.** Drop a Modrinth slug or a CurseForge id and the file lands in the right folder. `.mrpack` and `.zip` modpacks supported out of the box.
- **Tokens stay yours.** Microsoft and Azuriom secrets can't leak through logs or JSON dumps. OS keychain storage is one method call away.
- **You see everything.** Typed events stream every step on a broadcast bus — perfect for a real-time UI or telemetry.

---

## Quick Start

```toml
[dependencies]
lighty-launcher = { version = "26.5.1", features = ["vanilla"] }
tokio = { version = "1", features = ["full"] }
anyhow = "1.0"
```

```rust
use lighty_launcher::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("MyLauncher")?;

    let mut instance = VersionBuilder::new(
        "my-instance",
        Loader::Vanilla,
        "",
        "1.21.1",
    );

    let mut auth = OfflineAuth::new("Player123");
    let profile = auth.authenticate().await?;

    instance
        .launch(&profile, JavaDistribution::Temurin)
        .run()
        .await?;

    Ok(())
}
```

➡️  Want Microsoft auth, modpacks, event streams ?
Check the **[`examples/`](examples/)** folder — every loader, mod source and
auth provider has a runnable sample.

---

## Architecture

```
lighty-launcher/             # Root crate — prelude + feature gates
├── src/lib.rs
├── crates/
│   ├── core/                # AppState, HTTP client, hashing, extract
│   ├── auth/                # Offline / Microsoft / Azuriom (+ keyring opt-in)
│   ├── event/               # Broadcast bus + typed events
│   ├── java/                # JRE auto-download (Temurin/Zulu/Liberica/GraalVM)
│   ├── launch/              # Install orchestration + game process lifecycle
│   ├── loaders/             # Vanilla / Fabric / Quilt / Forge / NeoForge / ...
│   ├── modsloader/          # Modrinth + CurseForge clients + modpack pipelines
│   └── version/             # Fluent VersionBuilder
├── examples/                # Runnable samples (per-loader, mods/, modpacks/, auth/)
├── docs/                    # Cross-crate docs + assets
├── ASSETS_ROUTING.md        # Design: per-type asset routing
└── AUTH_SECRETS.md          # Design: token security model
```

---

## Documentation

📚 **Full documentation lives on GitBook**:
<https://hamadi.gitbook.io/lightylauncher>

The GitBook covers guides, walkthroughs, sequence diagrams and the
re-exports reference. The per-crate `README.md` linked below stays in the
repo as a quick API reference next to the code.

| Crate | Description |
|---|---|
| [`lighty-core`](crates/core/README.md) | App state, HTTP, hashing, archive extract |
| [`lighty-auth`](crates/auth/README.md) | Offline / Microsoft / Azuriom auth |
| [`lighty-event`](crates/event/README.md) | Broadcast event bus + typed events |
| [`lighty-java`](crates/java/README.md) | JRE download and discovery |
| [`lighty-launch`](crates/launch/README.md) | Install orchestrator + game runner |
| [`lighty-loaders`](crates/loaders/README.md) | Minecraft loader implementations |
| [`lighty-modsloader`](crates/modsloader/docs/overview.md) | Mod sources + modpack parsers |
| [`lighty-version`](crates/version/README.md) | Fluent VersionBuilder |

---

## Cargo Features

```toml
# Minimal — vanilla offline
lighty-launcher = { version = "26.5.1", features = ["vanilla"] }

# Fabric + Modrinth mods + live progress events
lighty-launcher = { version = "26.5.1", features = ["fabric", "modrinth", "events"] }

# Everything
lighty-launcher = { version = "26.5.1", features = ["all-loaders", "all-mods", "events", "tracing"] }
```

| Feature | Effect |
|---|---|
| `vanilla` / `fabric` / `quilt` / `neoforge` / `forge` | Enable that Minecraft loader (`forge` covers modern + legacy 1.7.10–1.12.2) |
| `lighty_updater` | Custom updater backend (auto-pulls vanilla/fabric/quilt/neoforge/forge) |
| `all-loaders` | Every loader above |
| `modrinth` | Modrinth API client + `.mrpack` modpack support |
| `curseforge` | CurseForge API client + `.zip` modpack support (requires API key) |
| `all-mods` | Both `modrinth` and `curseforge` |
| `events` | Typed broadcast events (`LaunchEvent`, `ModloaderEvent`, …) |
| `keyring` | Opt-in OS-keychain storage for auth tokens (see [`AUTH_SECRETS.md`](AUTH_SECRETS.md)) |
| `tracing` | Structured logging via the `tracing` crate |

---

## Contributing

PRs welcome. See the [Contributing Guide](CONTRIBUTING.md).

## License

[MIT](LICENSE).

## Related Projects

- **[LightyUpdater](https://github.com/Lighty-Launcher/LightyUpdater)** — companion server for custom modpack distribution.
