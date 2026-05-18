# macros — directory + tracing helpers

A handful of `#[macro_export]` macros auto-imported via `use
lighty_core::*;` or directly by name (`use lighty_core::mkdir;`).

## Directory helpers

All are async-aware: they use `tokio::fs::create_dir_all` under the
hood.

```rust
use lighty_core::{mkdir, try_mkdir, join_and_mkdir, join_and_mkdir_vec};
use std::path::Path;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let base = Path::new("/tmp/launcher");

    // Fire-and-forget — logs errors via trace_error!, swallows them
    mkdir!(base);

    // Propagates io::Error — use when failure should stop the pipeline
    try_mkdir!(base)?;

    // Join then mkdir, returns PathBuf
    let mods = join_and_mkdir!(base, "instances/foo/mods");

    // Sequence of joins, each level created
    let world = join_and_mkdir_vec!(base, &["instances", "foo", "saves", "world1"]);

    println!("{} / {}", mods.display(), world.display());
    Ok(())
}
```

`mkdir_blocking!` exists for non-async contexts and runs the
`create_dir_all` inside `spawn_blocking`.

## Tracing macros

```rust
use lighty_core::{trace_debug, trace_info, trace_warn, trace_error};

fn process(path: &str) {
    trace_info!("processing {}", path);
    if path.is_empty() {
        trace_warn!("empty path");
        return;
    }
    trace_debug!("ok");
}
```

Structured fields work too (forwarded to `tracing::*!`):

```rust
use lighty_core::trace_info;
trace_info!(
    version  = "1.21.1",
    loader   = "fabric",
    "launching instance"
);
```

Without the `tracing` Cargo feature, all four macros and `time_it!`
expand to no-ops — no runtime cost.

## `time_it!`

Logs the elapsed `Duration` of any expression at `DEBUG` level:

```rust
use lighty_core::time_it;

# fn expensive() -> i32 { 42 }
let result = time_it!("expensive op", expensive());
# let _ = result;
```

Output (with tracing on):
```
DEBUG label="expensive op" elapsed=12.3ms "Operation completed"
```

## Imports

The macros are exported at the crate root, so either of these works:

```rust
use lighty_core::{mkdir, trace_info};
// or
use lighty_core::*;
```

## See also

- [`how-to-use.md`](./how-to-use.md) — full walkthrough
- [`tracing` crate docs](https://docs.rs/tracing) — what the tracing
  macros expand into when the feature is on
