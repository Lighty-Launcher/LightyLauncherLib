# Module layout

Source files under `crates/event/src/module/`:

```
module/
├── auth.rs        AuthEvent
├── core.rs        CoreEvent
├── java.rs        JavaEvent
├── launch.rs      LaunchEvent
├── loader.rs      LoaderEvent
├── modloader.rs   ModloaderEvent  (split out of launch.rs)
└── console.rs     ConsoleStream, InstanceLaunchedEvent,
                   InstanceWindowAppearedEvent, InstanceExitedEvent,
                   ConsoleOutputEvent, InstanceDeletedEvent
```

Each `*.rs` defines one event family. The crate root re-exports them
flat (`pub use module::AuthEvent;` …) and wraps them in the root
`Event` enum.

`modloader.rs` is recent: the `ModResolve*` and `Modpack*` variants
used to live in `launch.rs` and are now their own `ModloaderEvent`
enum, reached through `Event::Modloader(_)`. The per-bucket
post-install summaries (`ResourcePacksInstalled`, `ShaderPacksInstalled`,
`DatapacksInstalled`) only live here.

## See also

- [`events.md`](./events.md) — catalogue of every variant per module
- [`architecture.md`](./architecture.md) — broadcast / fan-out model
