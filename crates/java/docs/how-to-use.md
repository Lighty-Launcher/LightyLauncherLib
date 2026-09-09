# Using lighty-java

Three operations: find a JRE on disk, install one if missing, run a
Java process.

## 1. Find or install a JRE

```rust
use lighty_core::AppState;
use lighty_java::{JavaDistribution, jre_downloader};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("LightyLauncher")?;
    let runtimes = AppState::cache_dir().join("java");

    // 1. Try to find an existing install first
    let java = match jre_downloader::find_java_binary(
        &runtimes,
        &JavaDistribution::Temurin,
        &21,
    ).await {
        Ok(path) => path,
        Err(_) => jre_downloader::jre_download(
            &runtimes,
            &JavaDistribution::Temurin,
            &21,
            |current, total| {
                if total > 0 {
                    println!("{:.1}%", current as f64 * 100.0 / total as f64);
                }
            },
            #[cfg(feature = "events")] None,
        ).await?,
    };

    println!("java: {}", java.display());
    Ok(())
}
```

`find_java_binary` uses `JavaDistribution::get_fallback` so it returns
the same path `jre_download` would have installed — no extra work to
correlate the two.

## 2. Spawn a Java process

```rust
use lighty_java::runtime::JavaRuntime;
use tokio::io::{AsyncBufReadExt, BufReader};
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let runtime = JavaRuntime::new("/usr/bin/java".into());

    let mut child = runtime.execute(
        vec![
            "-Xmx2G".into(),
            "-jar".into(),
            "app.jar".into(),
        ],
        Path::new("/tmp"),
    ).await?;

    // Stream output until the process exits
    let stdout = child.stdout.take().expect("stdout is piped");
    let mut lines = BufReader::new(stdout).lines();
    while let Some(line) = lines.next_line().await? {
        println!("{line}");
    }

    Ok(())
}
```

`execute` always pipes stdout and stderr, so both have to be drained:
an unread pipe fills up and the JVM blocks on `write`. Send the one you
don't care about to `tokio::io::sink()` rather than leaving it
untouched.

## 3. Subscribe to install events

```rust
use lighty_event::{Event, EventBus, JavaEvent};
use lighty_core::AppState;
use lighty_java::{JavaDistribution, jre_downloader};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AppState::init("LightyLauncher")?;

    #[cfg(feature = "events")]
    {
        let bus = EventBus::new(1000);
        let mut rx = bus.subscribe();
        tokio::spawn(async move {
            while let Ok(event) = rx.next().await {
                if let Event::Java(je) = event {
                    println!("{:?}", je);
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
    }
    Ok(())
}
```

The eight `JavaEvent` variants are listed in [`events.md`](./events.md).

## Distribution fallback

`JavaDistribution::get_fallback(version)` returns a replacement when
the chosen provider doesn't publish a given combo:

| Combination | Original | Fallback chain |
|---|---|---|
| Temurin Java 8 on macOS aarch64 | Temurin | Zulu → Liberica → Temurin |
| GraalVM Java 11 (any OS) | GraalVM | Zulu → Liberica → Temurin |
| Everything else | (unchanged) | — |

`jre_download` and `find_java_binary` consume that mapping internally,
so calling code always sees a working binary path.

## See also

- [`distributions.md`](./distributions.md) — provider matrix
- [`installation.md`](./installation.md) — download walkthrough
- [`runtime.md`](./runtime.md) — process spawn + I/O streaming
- [`events.md`](./events.md) — `JavaEvent` variants
