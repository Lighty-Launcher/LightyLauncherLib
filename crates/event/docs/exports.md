# Exports

## Root re-exports

```rust,ignore
pub use lighty_event::{
    // Bus
    EventBus, EventReceiver, EVENT_BUS,

    // Root enum
    Event,

    // Module enums
    AuthEvent, CoreEvent, JavaEvent,
    LaunchEvent, LoaderEvent, ModloaderEvent,

    // Console + instance events
    ConsoleOutputEvent, ConsoleStream,
    InstanceDeletedEvent, InstanceExitedEvent,
    InstanceLaunchedEvent, InstanceWindowAppearedEvent,

    // Errors
    EventReceiveError,    EventReceiveResult,
    EventSendError,       EventSendResult,
    EventTryReceiveError, EventTryReceiveResult,
};
```

## Bus

```rust,ignore
pub struct EventBus { /* clone-able sender wrapper */ }

impl EventBus {
    pub fn new(capacity: usize) -> Self;
    pub fn subscribe(&self)     -> EventReceiver;
    pub fn emit(&self, event: Event);                  // silently drops if no subscribers
}

pub struct EventReceiver { /* private */ }

impl EventReceiver {
    pub async fn next(&mut self)     -> EventReceiveResult<Event>;
    pub fn       try_next(&mut self) -> EventTryReceiveResult<Event>;
}

pub static EVENT_BUS: Lazy<EventBus>;     // capacity = 1000
```

## Module enums

Each lives under `lighty_event::module::<name>::<Enum>` and is
re-exported flat. Variant lists are in [`events.md`](./events.md).

| Crate that emits | Enum |
|---|---|
| `lighty-auth`        | `AuthEvent` |
| `lighty-core`        | `CoreEvent` |
| `lighty-java`        | `JavaEvent` |
| `lighty-launch`      | `LaunchEvent` |
| `lighty-loaders`     | `LoaderEvent` |
| `lighty-modsloader` + `lighty-launch` | `ModloaderEvent` |

## Console + instance events

Defined as structs (not variants) in `module/console.rs`. They carry
a `SystemTime` timestamp serialized as Unix seconds:

- `InstanceLaunchedEvent { pid, instance_name, version, username, timestamp }`
- `InstanceWindowAppearedEvent { pid, instance_name, version, timestamp }`
- `InstanceExitedEvent { pid, instance_name, exit_code: Option<i32>, timestamp }`
- `ConsoleOutputEvent { pid, instance_name, stream: ConsoleStream, line, timestamp }`
- `InstanceDeletedEvent { instance_name, timestamp }`
- `ConsoleStream { Stdout, Stderr }`

## Errors

```rust,ignore
pub enum EventReceiveError {
    Closed,                                // bus dropped
    Lagged(u64),                           // missed N events
}
pub enum EventTryReceiveError {
    Empty,
    Closed,
    Lagged(u64),
}
pub enum EventSendError {
    NoReceivers,
}
```

## See also

- [`events.md`](./events.md) — variant catalogue
- [`how-to-use.md`](./how-to-use.md) — wiring a subscriber
