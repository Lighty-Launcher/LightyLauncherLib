# CoreEvent

Three progress events emitted by `lighty_core::extract::*` when the
`events` feature is on. They surface through the global event bus as
`Event::Core(CoreEvent::…)`.

Variants (defined in `crates/event/src/module/core.rs`):

| Variant | Fields | When |
|---|---|---|
| `ExtractionStarted` | `archive_type: String, file_count: usize, destination: String` | First call inside `zip_extract` / `tar_gz_extract`; for tar.gz the `file_count` is `0` because the format is streamed |
| `ExtractionProgress` | `files_extracted: usize, total_files: usize` | Every 10 entries (and on the final entry) for zip; every 10 entries for tar.gz |
| `ExtractionCompleted` | `archive_type: String, files_extracted: usize` | Once the last entry has been written |

`archive_type` is the string `"ZIP"` or `"TAR.GZ"`.

## Subscribing

```rust,no_run
use lighty_core::{AppState, extract::zip_extract};
use lighty_event::{Event, CoreEvent, EventBus};
use tokio::{fs::File, io::BufReader};
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("LightyLauncher")?;
    let bus = EventBus::new(1000);
    let mut rx = bus.subscribe();

    tokio::spawn(async move {
        while let Ok(event) = rx.next().await {
            if let Event::Core(ce) = event {
                match ce {
                    CoreEvent::ExtractionStarted { archive_type, file_count, .. } =>
                        println!("starting {archive_type} ({file_count} files)"),
                    CoreEvent::ExtractionProgress { files_extracted, total_files } =>
                        println!("  {files_extracted}/{total_files}"),
                    CoreEvent::ExtractionCompleted { files_extracted, .. } =>
                        println!("done: {files_extracted} files"),
                }
            }
        }
    });

    let archive = BufReader::new(File::open("mod.zip").await?);
    zip_extract(archive, Path::new("./out"), Some(&bus)).await?;
    Ok(())
}
```

## See also

- [`extract.md`](./extract.md) — the functions that emit these events
- [`../../event/docs/events.md`](../../event/docs/events.md) — full
  catalogue of every event in the workspace
