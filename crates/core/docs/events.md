# Events

## Overview

`lighty-core` emits `CoreEvent` types through the event bus system provided by `lighty-event`. These events track extraction operations progress.

**Feature**: Requires `events` feature flag

**Export**:
- Event types: `lighty_event::CoreEvent`
- Re-export: `lighty_launcher::event::CoreEvent`

## CoreEvent Types

### ExtractionStarted

Emitted when archive extraction begins.

**Fields**:
- `archive_type: String` - Archive format ("ZIP", "TAR", "TAR.GZ")
- `file_count: usize` - Total number of files to extract
- `destination: String` - Output directory path

**When emitted**: At the start of `zip_extract`, `tar_extract`, or `tar_gz_extract`

**Example**:
```rust
use lighty_event::{EventBus, Event, CoreEvent};
use lighty_core::extract::zip_extract;
use tokio::fs::File;
use tokio::io::BufReader;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    lighty_core::AppState::init("MyLauncher")?;

    let event_bus = EventBus::new(1000);
    let mut receiver = event_bus.subscribe();

    tokio::spawn(async move {
        while let Ok(event) = receiver.next().await {
            if let Event::Core(CoreEvent::ExtractionStarted { archive_type, file_count, destination }) = event {
                println!("Starting {} extraction:", archive_type);
                println!("  Files: {}", file_count);
                println!("  Destination: {}", destination);
            }
        }
    });

    let file = File::open("archive.zip").await?;
    let reader = BufReader::new(file);
    zip_extract(reader, "output", Some(&event_bus)).await?;

    Ok(())
}
```

### FileExtracted

Emitted after each file is successfully extracted from the archive.

**Fields**:
- `file_name: String` - Name of the extracted file
- `index: usize` - Current file index (0-based)
- `total: usize` - Total number of files

**When emitted**: After each file write operation completes

**Example**:
```rust
use lighty_event::{EventBus, Event, CoreEvent};
use lighty_core::extract::zip_extract;
use tokio::fs::File;
use tokio::io::BufReader;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    lighty_core::AppState::init("MyLauncher")?;

    let event_bus = EventBus::new(1000);
    let mut receiver = event_bus.subscribe();

    tokio::spawn(async move {
        while let Ok(event) = receiver.next().await {
            if let Event::Core(CoreEvent::FileExtracted { file_name, index, total }) = event {
                let progress = ((index + 1) as f64 / total as f64) * 100.0;
                println!("[{:.1}%] Extracted: {}", progress, file_name);
            }
        }
    });

    let file = File::open("archive.zip").await?;
    let reader = BufReader::new(file);
    zip_extract(reader, "output", Some(&event_bus)).await?;

    Ok(())
}
```

### ExtractionComplete

Emitted when all files have been successfully extracted.

**Fields**:
- `file_count: usize` - Total number of files extracted

**When emitted**: After the last file is extracted and all operations complete

**Example**:
```rust
use lighty_event::{EventBus, Event, CoreEvent};
use lighty_core::extract::tar_gz_extract;
use tokio::fs::File;
use tokio::io::BufReader;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    lighty_core::AppState::init("MyLauncher")?;

    let event_bus = EventBus::new(1000);
    let mut receiver = event_bus.subscribe();

    tokio::spawn(async move {
        while let Ok(event) = receiver.next().await {
            if let Event::Core(CoreEvent::ExtractionComplete { file_count }) = event {
                println!("✓ Extraction complete!");
                println!("  Total files: {}", file_count);
            }
        }
    });

    let file = File::open("java.tar.gz").await?;
    let reader = BufReader::new(file);
    tar_gz_extract(reader, "java", Some(&event_bus)).await?;

    Ok(())
}
```

## Complete Event Flow

```rust
use lighty_event::{EventBus, Event, CoreEvent};
use lighty_core::extract::zip_extract;
use tokio::fs::File;
use tokio::io::BufReader;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    lighty_core::AppState::init("MyLauncher")?;

    let event_bus = EventBus::new(1000);
    let mut receiver = event_bus.subscribe();

    tokio::spawn(async move {
        while let Ok(event) = receiver.next().await {
            match event {
                Event::Core(CoreEvent::ExtractionStarted { archive_type, file_count, destination }) => {
                    println!("🗜️  Extracting {} archive", archive_type);
                    println!("   {} files → {}", file_count, destination);
                }
                Event::Core(CoreEvent::FileExtracted { file_name, index, total }) => {
                    let progress = ((index + 1) as f64 / total as f64) * 100.0;
                    println!("   [{:>3.0}%] {}", progress, file_name);
                }
                Event::Core(CoreEvent::ExtractionComplete { file_count }) => {
                    println!("✓  Complete! Extracted {} files", file_count);
                }
                _ => {}
            }
        }
    });

    let file = File::open("mods.zip").await?;
    let reader = BufReader::new(file);
    zip_extract(reader, "instance/mods", Some(&event_bus)).await?;

    // Allow events to be processed
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    Ok(())
}
```

**Output**:
```
🗜️  Extracting ZIP archive
   42 files → instance/mods
   [  2%] fabric-api-0.92.0.jar
   [  5%] sodium-0.5.8.jar
   [  7%] iris-1.7.0.jar
   ... (38 more files)
   [100%] config.json
✓  Complete! Extracted 42 files
```

## Event Timing

### Extraction Flow

```
ExtractionStarted
    ↓
FileExtracted (file 0)
    ↓
FileExtracted (file 1)
    ↓
FileExtracted (file 2)
    ↓
... (for each file)
    ↓
FileExtracted (file N-1)
    ↓
ExtractionComplete
```

## Integration with Other Crates

### lighty-java

Uses `CoreEvent` for tracking Java runtime extraction:

```rust
// Inside lighty-java
use lighty_event::{EVENT_BUS, Event, CoreEvent};
use lighty_core::extract::tar_gz_extract;

// Java distribution extraction emits CoreEvent
tar_gz_extract(archive, java_dir, Some(&EVENT_BUS)).await?;
// → Emits ExtractionStarted, FileExtracted, ExtractionComplete
```

### lighty-loaders

May use extraction for mod archives:

```rust
// Extracting mod archives
use lighty_core::extract::zip_extract;
use lighty_event::EventBus;

let event_bus = EventBus::new(1000);
zip_extract(mod_archive, mods_dir, Some(&event_bus)).await?;
```

## Exports

**Event types**:
```rust
// In lighty-event
use lighty_event::CoreEvent;

// Re-exported in lighty-launcher
use lighty_launcher::event::CoreEvent;
```

**Core extraction functions** (from lighty-core):
```rust
use lighty_core::extract::{zip_extract, tar_extract, tar_gz_extract};
```

## Related Documentation

- [How to Use](./how-to-use.md) - Practical extraction examples
- [Extract](./extract.md) - Detailed extraction system guide
- [lighty-event Events](../../event/docs/events.md) - All event types
- [Exports](./exports.md) - Complete export reference
