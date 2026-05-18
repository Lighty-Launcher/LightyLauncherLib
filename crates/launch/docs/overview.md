# Overview

`lighty-launch` is the orchestrator that turns a fully-described
instance (a `VersionBuilder` or any `VersionInfo` impl) plus a
`UserProfile` into a running Minecraft process. It owns the install
pipeline, builds the argv, spawns the JVM and tracks the resulting
process across its lifetime.

## What this crate gives you

| Item | What it is |
|---|---|
| [`Launch`](./launch.md) trait | Adds `.launch(&profile, java)` to any installable instance |
| [`LaunchBuilder`](./launch.md#launchbuilder-api) | Fluent builder: JVM options, game args, event bus, then `.run().await` |
| [`Installer`](./installation.md) trait | 8-way parallel install: libraries, natives, client, assets, mods, resource/shader/datapacks (+ optional modpack pre-step) |
| [`Arguments`](./arguments.md) trait | Builds the JVM + game argv from a `Version` plus per-launch overrides |
| [`InstanceControl`](./instance-control.md) trait | PID tracking, close, delete, size breakdown |

The launch pipeline relies on canonical concepts owned by other crates
and just cross-references them — see the table below.

## The pipeline at a glance

```text
.launch(profile, java)
    │
    ▼
1. Fetch loader metadata        (lighty-loaders dispatch)
2. Ensure Java present          (lighty-java; downloads if missing)
3. Install — 8 buckets in        ─► see installation.md
   parallel + optional modpack
4. Forge / NeoForge post-hook    ─► run install_profile processors
5. Build argv + spawn JVM        ─► see arguments.md + launch.md
6. Register PID + stream stdio   ─► see instance-control.md
```

Each step emits an event when the `events` feature is on — see
[events.md](./events.md).

## Canonical concepts (cross-refs)

| Concept | Lives in |
|---|---|
| `AppState::init` + OS paths | [`crates/core/docs/app_state.md`](../../core/docs/app_state.md) |
| `VersionBuilder` / `LightyVersionBuilder` fluent API | [`crates/version/docs/how-to-use.md`](../../version/docs/how-to-use.md) |
| Loader registry + per-loader install details | [`crates/loaders/docs/overview.md`](../../loaders/docs/overview.md) and `loaders/<name>.md` siblings |
| Modrinth / CurseForge clients, modpack pipeline | [`crates/modsloader/docs/`](../../modsloader/docs/) |
| Cross-module event catalogue | [`crates/event/docs/events.md`](../../event/docs/events.md) |

## Cargo features

| Feature | Adds |
|---|---|
| `events` | `LaunchEvent` + `ModloaderEvent` emission, `with_event_bus(...)` on the builder, console streaming through events |
| `keyring` | Lets the argv builder read `--accessToken` from a `TokenHandle` (forwarded to `lighty-auth/keyring`) |
| `forge`, `neoforge` | Pulls the matching loader's post-install processors. `forge` covers both modern Forge **and** legacy 1.7.10 – 1.12.2 in a single feature |
| `fabric`, `quilt` | Loader support (no extra install step) |
| `modrinth`, `curseforge` | Activates the user-mod resolver + the matching modpack format parser |
| `tracing` | Provider-level `tracing` logs |

## Related

- [How to use](./how-to-use.md) — the common `.launch().run()` pattern
- [Launch](./launch.md) — pipeline detail, `Launch` trait, builder
- [Installation](./installation.md) — 8-bucket pipeline, SHA1 checks, modpack
- [Instance lifecycle](./instance-lifecycle.md) — manager, console, exit
- [Instance control](./instance-control.md) — `InstanceControl` API
- [Arguments](./arguments.md) — placeholders, JVM, game args, token routing
- [Events](./events.md) — `LaunchEvent` + cross-ref to `ModloaderEvent`
- [Exports](./exports.md) — full public surface
