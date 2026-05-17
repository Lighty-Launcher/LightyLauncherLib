# lighty-auth

Multi-provider authentication system for Minecraft launchers with OAuth2 and CMS integrations.

## Overview

**Version**: 26.5.1
**Part of**: [LightyLauncher](https://crates.io/crates/lighty-launcher)

`lighty-auth` provides a unified trait-based authentication system supporting multiple providers:
- **Offline Mode** - Local authentication without network, generates deterministic UUIDs
- **Microsoft Account** - OAuth 2.0 Device Code Flow via Microsoft/Xbox Live/Minecraft Services
- **Azuriom CMS** - Server authentication with 2FA support, roles, and permissions
- **Custom Providers** - Implement the `Authenticator` trait for your own auth system

## Quick Start

```toml
[dependencies]
lighty-auth = "26.5.1"
```

### Offline Authentication

```rust
use lighty_auth::{offline::OfflineAuth, Authenticator};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut auth = OfflineAuth::new("PlayerName");

    #[cfg(feature = "events")]
    let profile = auth.authenticate(None).await?;

    #[cfg(not(feature = "events"))]
    let profile = auth.authenticate().await?;

    println!("Username: {}", profile.username);
    println!("UUID: {}", profile.uuid);

    Ok(())
}
```

### Microsoft Authentication

```rust
use lighty_auth::{microsoft::MicrosoftAuth, Authenticator};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut auth = MicrosoftAuth::new("your-azure-client-id");

    // Display device code to user
    auth.set_device_code_callback(|code, url| {
        println!("Please visit: {}", url);
        println!("And enter code: {}", code);
    });

    #[cfg(feature = "events")]
    let profile = auth.authenticate(None).await?;

    #[cfg(not(feature = "events"))]
    let profile = auth.authenticate().await?;

    println!("Logged in as: {}", profile.username);

    Ok(())
}
```

## Authentication Providers

| Provider | Status | Network | Use Cases |
|----------|--------|---------|-----------|
| **Offline** | ✅ Stable | Not required | Testing, offline play, development |
| **Microsoft** | ✅ Stable | Required | Legitimate Minecraft accounts |
| **Azuriom** | ✅ Stable | Required | Custom server authentication, launcher whitelisting |

## Features

- **Trait-based** - Implement `Authenticator` for custom providers
- **Event integration** - Track authentication progress with events
- **2FA support** - Azuriom two-factor authentication
- **Offline UUIDs** - Deterministic UUID generation
- **Role system** - User roles with colors and permissions

## Documentation

📚 **[Complete Documentation](./docs)**

| Guide | Description |
|-------|-------------|
| [How to Use](./docs/how-to-use.md) | Practical authentication guide with examples |
| [Overview](./docs/overview.md) | Architecture and design patterns |
| [Exports](./docs/exports.md) | Complete export reference |
| [Events](./docs/events.md) | AuthEvent types |
| [Offline](./docs/offline.md) | Offline mode and UUID generation |
| [Microsoft](./docs/microsoft.md) | Microsoft OAuth2 flow |
| [Azuriom](./docs/azuriom.md) | Azuriom CMS authentication |
| [Trait](./docs/trait.md) | Implementing custom Authenticator |

## Related Crates

- **[lighty-launcher](../../../README.md)** - Main package
- **[lighty-event](../event/README.md)** - Event system (for AuthEvent)
- **[lighty-core](../core/README.md)** - Hash utilities for offline UUID
- **[lighty-launch](../launch/README.md)** - Uses UserProfile for launching

## License

MIT
