# lighty-core

Foundation crate for the LightyLauncher workspace. Holds the pieces
every other crate ends up needing: process-wide paths, the shared HTTP
client, a Tokio download helper, archive extraction with traversal
protection, SHA1 hashing, OS / architecture detection, and a small set
of `tracing`-aware macros.

## What's inside

| Module       | Purpose                                                                 |
|--------------|-------------------------------------------------------------------------|
| `app_state`  | `AppState::init` + the per-launcher data / config / cache directories. |
| `hosts`      | `HTTP_CLIENT` — a single `reqwest::Client` shared by the whole workspace. |
| `download`   | `download_file_untracked` (one-shot) and `download_file` (with progress callback). |
| `extract`    | `zip_extract` and `tar_gz_extract` with path-traversal, symlink and 2 GB bomb protection. |
| `hash`       | SHA1 helpers (sync + async, streaming + in-memory, raw bytes + hex). |
| `system`     | `OS`, `ARCHITECTURE` consts plus per-vendor name mapping (Mojang, Adoptium, Azul, Graal). |
| `macros`     | `mkdir!`, `join_and_mkdir!`, `trace_debug!`, `trace_info!`, `trace_warn!`, `trace_error!`, `time_it!`. |
| `errors`     | `AppStateError`, `DownloadError`, `ExtractError`, `HashError`, `SystemError`, `QueryError`. |

The crate is **not** Minecraft-specific — it could host the same
helpers for any launcher.

## Cargo features

| Feature   | Effect                                                                                  |
|-----------|-----------------------------------------------------------------------------------------|
| `events`  | `zip_extract` / `tar_gz_extract` gain an `Option<&EventBus>` arg and emit `CoreEvent`. |
| `tracing` | Activates the `trace_*!` macros (they expand to no-ops otherwise).                      |

`QueryError` lives here because every layer above (loaders, modsloader,
launch) returns it — keeping it in the root crate avoids a circular
dependency.

## How the pieces fit together

```mermaid
flowchart LR
    APP[Your code] --> AS[AppState]
    APP --> DL[download]
    APP --> EX[extract]
    APP --> HS[hash]
    APP --> SYS[system]
    DL --> HTTP[hosts::HTTP_CLIENT]
    DL -.optional.-> EX
    EX -.feature "events".-> BUS[lighty_event::EventBus]
```

Most flows look like *download → verify SHA1 → extract*. The launcher
wires that pipeline once in `lighty-launch`; `lighty-java` does the
same for JRE archives.

## Design notes

- **Zero hidden state.** Only `AppState` and `HTTP_CLIENT` are
  process-globals; everything else takes its inputs by argument.
- **Streaming I/O.** Downloads chunk-by-chunk; extraction copies
  through 256 KiB buffers; hashing has a streaming variant for files
  bigger than a few MB.
- **Hard limits.** Archives reject entries above 2 GiB and any path
  that resolves outside the destination directory.

## See also

- [`app_state.md`](./app_state.md) — canonical reference for the path layout
- [`how-to-use.md`](./how-to-use.md) — one runnable example per feature
- [`download.md`](./download.md), [`extract.md`](./extract.md),
  [`hash.md`](./hash.md), [`system.md`](./system.md),
  [`macros.md`](./macros.md) — module-by-module detail
- [`events.md`](./events.md) — the three `CoreEvent` variants
- [`../../event/docs/events.md`](../../event/docs/events.md) — full event catalogue
