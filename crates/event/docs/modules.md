# Event Module System

## Overview

Events are organized into modules for better maintainability.

## Module Structure

```
module/
├── auth.rs       - Authentication events
├── java.rs       - Java distribution events
├── launch.rs     - Game launch + install lifecycle events
├── loader.rs     - Mod loader metadata events
├── modloader.rs  - Mod-source / modpack / mod-like bucket events
├── core.rs       - Core system events
└── console.rs    - Instance console events
```

`modloader.rs` is a recent split: the `ModResolve*` and `Modpack*`
variants used to live in `launch.rs` but are now their own enum
(`ModloaderEvent`), reachable through `Event::Modloader(_)`. The
three per-bucket summaries `ResourcePacksInstalled`,
`ShaderPacksInstalled` and `DatapacksInstalled` were added at the same
time and only live here.

## Creating Custom Modules

See main repository for extension guides.
