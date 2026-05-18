# Instance lifecycle

This page documents what happens behind `InstanceControl`: the
[`InstanceManager`](#instance-manager) singleton, the launch-to-exit
state machine, the console streaming task. For the **public API**
(`get_pid` / `close_instance` / `delete_instance` / `size_of_instance`),
see [instance-control.md](./instance-control.md).

## State machine

```text
1. Launch
   ├─ JVM spawned (Command::spawn)
   ├─ PID captured (child.id())
   ├─ GameInstance registered in INSTANCE_MANAGER
   └─ LaunchEvent::Launched { pid } emitted

2. Running
   ├─ stdout / stderr streamed → LaunchEvent::ProcessOutput { pid, stream, line }
   └─ window-appearance watcher (events feature) emits a one-shot event
      once the OS detects a window for the PID

3. Exit (one of):
   a. Process terminates on its own
      ├─ child.wait().await resolves with exit code
      ├─ LaunchEvent::ProcessExited { pid, exit_code } emitted
      └─ INSTANCE_MANAGER.unregister_instance(pid)
   b. User calls InstanceControl::close_instance(pid)
      ├─ INSTANCE_MANAGER removes the entry up-front
      ├─ SIGTERM (Unix) / taskkill /F (Windows) sent
      └─ Console task observes the child exiting and also
         emits ProcessExited (idempotent unregister)

4. Optional cleanup
   └─ InstanceControl::delete_instance() — only legal once no PID is
      tracked (otherwise InstanceError::StillRunning)
```

## Instance manager

Global singleton:

```rust,ignore
pub(crate) static INSTANCE_MANAGER: Lazy<InstanceManager> = Lazy::new(InstanceManager::new);

pub(crate) struct InstanceManager {
    instances: RwLock<HashMap<u32, GameInstance>>,
}

pub(crate) struct GameInstance {
    pub pid:           u32,
    pub instance_name: String,
    pub version:       String,    // "{mc}-{loader}"
    pub username:      String,
    pub game_dir:      PathBuf,
    pub started_at:    SystemTime,
}
```

Keyed by PID — that's how the console task and `close_instance` find
each other. Lookups by instance name (`get_pid`, `get_pids`) iterate
the map and filter by `instance_name`.

### Operations

| Operation | Trigger |
|---|---|
| `register_instance(GameInstance)` | The launch runner after `spawn()`. Returns `InstanceError::DuplicatePid` if the PID is already tracked (race or OS PID reuse). |
| `unregister_instance(pid)` | Console task after `child.wait()` resolves, **and** `close_instance` up-front (idempotent). |
| `get_pid(name)` / `get_pids(name)` | Public API used by `InstanceControl`. |
| `is_alive(pid)` | Internal — tells callers whether a PID is still tracked. |
| `close_instance(pid)` | Public API; removes the entry, then sends the platform kill signal. |

A poisoned `RwLock` is recovered with `PoisonError::into_inner` — the
manager never panics on lock contention.

## Console streaming

When the JVM is spawned, the runner hands the `Child` to a dedicated
tokio task — `handle_console_streams(pid, name, child, bus)`. The task:

1. Takes ownership of `child.stdout` and `child.stderr`.
2. Spawns one inner task per stream that reads line-by-line and emits
   `LaunchEvent::ProcessOutput { pid, stream, line }` (with `stream`
   set to `"stdout"` or `"stderr"`).
3. `awaits` `child.wait()`.
4. Emits `LaunchEvent::ProcessExited { pid, exit_code }`.
5. Calls `INSTANCE_MANAGER.unregister_instance(pid)`.

Without the `events` feature, console lines are logged via the
`tracing` macros instead of being emitted as events. The lifecycle
itself is identical.

## Window-appearance watcher (events feature)

Spawned in parallel with the console task. Polls the OS until a window
is attached to the spawned PID, then emits a one-shot event so UIs can
flip from "Launching…" to "Game running". Implementation in
`crates/launch/src/launch/window.rs`.

## Why `SIGTERM` on Unix

Sending `SIGTERM` rather than `SIGKILL` lets the JVM run its
registered shutdown hooks before exit — which means Minecraft saves
the world, flushes the chunk cache, and exits cleanly. `taskkill /F`
on Windows is the closest equivalent that actually terminates the
process tree reliably; the trade-off is that no shutdown hooks fire
there, which matches the historical behaviour of every other launcher
on the platform.

## What `delete_instance` actually wipes

```text
{game_dirs()}/                      ← removed recursively
├── libraries/
├── natives/
├── assets/
├── versions/
├── mods/, resourcepacks/, …
└── saves/, screenshots/, options.txt
```

`java_dirs()` is **not** touched (Java distributions are shared across
instances). Cache directories under `{cache_dir}/modpacks/` are
preserved as well — they're keyed by URL SHA1 and benefit other
instances.

## Re-launching after close

`get_pid()` becomes `None` immediately when `close_instance` returns
(the manager entry is removed before the kill signal is sent). The
console task may still be wrapping up its emit of `ProcessExited` for
a few hundred milliseconds — for accurate "is it really gone" checks
in UIs, listen for `ProcessExited` rather than polling `get_pid`.

## Related

- [Instance control](./instance-control.md) — public API
- [Launch](./launch.md) — where the state machine starts (step 6 & 7)
- [Events](./events.md) — `Launched` / `ProcessOutput` / `ProcessExited`
- [Exports](./exports.md)
