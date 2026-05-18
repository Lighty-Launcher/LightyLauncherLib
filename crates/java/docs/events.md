# JavaEvent

Eight progress events emitted by `jre_downloader::jre_download` when
the `events` feature is on. They surface as
`Event::Java(JavaEvent::…)`.

Defined in `crates/event/src/module/java.rs`. The full workspace
event catalogue lives in
[`../../event/docs/events.md`](../../event/docs/events.md).

## Variants

| Variant | Fields | When |
|---|---|---|
| `JavaNotFound` | `distribution: String, version: u8` | (Reserved — not emitted by `jre_downloader`. Available for caller code.) |
| `JavaAlreadyInstalled` | `distribution, version, binary_path` | (Reserved — caller can emit before short-circuiting.) |
| `JavaDownloadStarted` | `distribution, version, total_bytes: u64` | After the URL resolves and `Content-Length` is known |
| `JavaDownloadProgress` | `bytes: u64` | Each chunk; `bytes` is the cumulative count |
| `JavaDownloadCompleted` | `distribution, version` | After the last chunk lands |
| `JavaExtractionStarted` | `distribution, version` | Before `zip_extract` / `tar_gz_extract` runs |
| `JavaExtractionProgress` | `files_extracted: usize, total_files: usize` | (Reserved — not currently emitted directly; extraction events come through `CoreEvent` instead.) |
| `JavaExtractionCompleted` | `distribution, version, binary_path: String` | After the binary is located on disk |

`distribution` is the canonical lowercase name (`"temurin"`,
`"graalvm"`, `"zulu"`, `"liberica"`).

The `Reserved` rows above exist in the enum so callers can build
richer state machines on top — `lighty-java` itself emits the five
`*Started / *Progress / *Completed` pairs.

## Subscriber

```rust,no_run
use lighty_core::AppState;
use lighty_event::{Event, EventBus, JavaEvent};
use lighty_java::{JavaDistribution, jre_downloader};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("LightyLauncher")?;

    let bus = EventBus::new(1000);
    let mut rx = bus.subscribe();

    tokio::spawn(async move {
        while let Ok(event) = rx.next().await {
            if let Event::Java(je) = event {
                match je {
                    JavaEvent::JavaDownloadStarted { distribution, version, total_bytes } =>
                        println!("downloading {distribution} {version} ({total_bytes} bytes)"),
                    JavaEvent::JavaDownloadProgress { bytes } =>
                        print!("\r{} MB", bytes / 1_048_576),
                    JavaEvent::JavaDownloadCompleted { distribution, version } =>
                        println!("\ndownloaded {distribution} {version}"),
                    JavaEvent::JavaExtractionStarted { distribution, version } =>
                        println!("extracting {distribution} {version}"),
                    JavaEvent::JavaExtractionCompleted { binary_path, .. } =>
                        println!("ready: {binary_path}"),
                    _ => {}
                }
            }
        }
    });

    jre_downloader::jre_download(
        &AppState::cache_dir().join("java"),
        &JavaDistribution::Temurin,
        &21,
        |_, _| {},
        Some(&bus),
    ).await?;
    Ok(())
}
```

## See also

- [`installation.md`](./installation.md) — the function that emits these
- [`../../event/docs/events.md`](../../event/docs/events.md) — full catalogue
