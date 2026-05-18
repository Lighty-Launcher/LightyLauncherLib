# Using lighty-event

Two patterns: pass your own `EventBus` into a builder (the common
case), or subscribe to the global `EVENT_BUS`.

## 1. Subscribe to a launch bus

`lighty-launch` accepts a bus via `.with_event_bus(&bus)`:

```rust
use lighty_event::{Event, EventBus, LaunchEvent, ConsoleStream};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let bus = EventBus::new(1000);
    let mut rx = bus.subscribe();

    tokio::spawn(async move {
        while let Ok(event) = rx.next().await {
            match event {
                Event::Launch(LaunchEvent::InstallProgress { bytes }) =>
                    println!("[install] {bytes} bytes"),
                Event::Launch(LaunchEvent::Launched { pid, .. }) =>
                    println!("[launch] pid {pid}"),
                Event::ConsoleOutput(e) => match e.stream {
                    ConsoleStream::Stdout => print!("[out] {}", e.line),
                    ConsoleStream::Stderr => eprint!("[err] {}", e.line),
                },
                _ => {}
            }
        }
    });

    // ... build a VersionBuilder and call .launch(...).with_event_bus(&bus).run().await
    Ok(())
}
```

Full event reference: [`events.md`](./events.md).

## 2. Subscribe to the global bus

Some crates emit through the global `EVENT_BUS` (extraction, modloader
resolver). Subscribe the same way:

```rust
use lighty_event::{EVENT_BUS, Event, CoreEvent};

#[tokio::main]
async fn main() {
    let mut rx = EVENT_BUS.subscribe();
    tokio::spawn(async move {
        while let Ok(event) = rx.next().await {
            if let Event::Core(CoreEvent::ExtractionProgress { files_extracted, total_files }) = event {
                println!("extract {files_extracted}/{total_files}");
            }
        }
    });
}
```

The bus capacity is 1000; if a subscriber lags more than that, the
next `next().await` returns
`EventReceiveError::Lagged(missed_count)`.

## 3. Fan out to several subscribers

```rust
use lighty_event::EventBus;

let bus = EventBus::new(1000);

let mut ui = bus.subscribe();
let mut log = bus.subscribe();
let mut analytics = bus.subscribe();

tokio::spawn(async move { while let Ok(e) = ui.next().await        { /* update UI */ } });
tokio::spawn(async move { while let Ok(e) = log.next().await       { /* write log */ } });
tokio::spawn(async move { while let Ok(e) = analytics.next().await { /* metrics */ } });
```

Every subscriber gets every event. No filtering on the bus side —
`match` in your loop.

## 4. Filter on a single variant

```rust
use lighty_event::{EventBus, Event, JavaEvent};

#[tokio::main]
async fn main() {
    let bus = EventBus::new(1000);
    let mut rx = bus.subscribe();

    tokio::spawn(async move {
        while let Ok(event) = rx.next().await {
            if let Event::Java(JavaEvent::JavaDownloadProgress { bytes }) = event {
                println!("[java] {} MB", bytes / 1_048_576);
            }
        }
    });
}
```

## 5. Stream console output

```rust
use lighty_event::{Event, EventBus, ConsoleStream};

#[tokio::main]
async fn main() {
    let bus = EventBus::new(1000);
    let mut rx = bus.subscribe();

    tokio::spawn(async move {
        while let Ok(event) = rx.next().await {
            if let Event::ConsoleOutput(e) = event {
                match e.stream {
                    ConsoleStream::Stdout => print!("[out] {}", e.line),
                    ConsoleStream::Stderr => eprint!("[err] {}", e.line),
                }
            }
        }
    });
}
```

## 6. Non-blocking peek

```rust
use lighty_event::{EventBus, EventTryReceiveError};

#[tokio::main]
async fn main() {
    let bus = EventBus::new(1000);
    let mut rx = bus.subscribe();
    loop {
        match rx.try_next() {
            Ok(event)                                 => { let _ = event; /* handle */ }
            Err(EventTryReceiveError::Empty)          => break,        // nothing waiting
            Err(EventTryReceiveError::Closed)         => break,
            Err(EventTryReceiveError::Lagged(missed)) => eprintln!("lagged {missed}"),
        }
    }
}
```

Use `try_next` from sync contexts (e.g. UI thread tick) or when you
want a single drain without `await`.

## See also

- [`events.md`](./events.md) — full catalogue + which crate emits what
- [`exports.md`](./exports.md) — `EventBus` / `EventReceiver` API
