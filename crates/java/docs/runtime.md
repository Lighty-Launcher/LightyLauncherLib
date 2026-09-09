# Runtime — spawn a `java` process

`JavaRuntime` is a thin wrapper around `tokio::process::Command`. It
spawns the binary and returns the `Child` with stdout / stderr piped,
leaving the caller free to stream them however it wants.

## API

```rust
pub struct JavaRuntime(pub PathBuf);

impl JavaRuntime {
    pub fn new(path: PathBuf) -> Self;

    pub async fn execute(
        &self,
        arguments: Vec<String>,
        game_dir: &Path,
    ) -> JavaRuntimeResult<tokio::process::Child>;
}
```

`game_dir` becomes the process's working directory (this is what gets
passed to Minecraft via `${game_directory}`).

Both pipes are **always** created, so whoever owns the `Child` has to
drain them: an unread pipe fills up and the JVM blocks on `write` once
the buffer is full. `lighty-launch` does this in
`handle_console_streams`.

Windows-only detail: `execute` sets `CREATE_NO_WINDOW` so spawned
processes don't pop up a console window.

## Examples

### Print `java -version`

```rust
use lighty_java::runtime::JavaRuntime;
use tokio::io::{AsyncBufReadExt, BufReader};
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let rt = JavaRuntime::new("/usr/bin/java".into());
    let mut child = rt.execute(vec!["-version".into()], Path::new(".")).await?;

    let stderr = child.stderr.take().expect("stderr is piped");
    let mut lines = BufReader::new(stderr).lines();
    while let Some(line) = lines.next_line().await? {
        eprintln!("{line}");
    }
    Ok(())
}
```

`java -version` writes to stderr — drain stdout too, or the pipe fills.

### Launch a JAR with memory tuning

```rust
use lighty_java::runtime::JavaRuntime;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let rt = JavaRuntime::new("/path/to/java".into());

    let args = vec![
        "-Xmx4G".into(),
        "-Xms1G".into(),
        "-XX:+UseG1GC".into(),
        "-jar".into(),
        "minecraft.jar".into(),
    ];
    let mut child = rt.execute(args, Path::new("/games/minecraft")).await?;

    let stdout = child.stdout.take().expect("stdout is piped");
    let mut stderr = child.stderr.take().expect("stderr is piped");
    tokio::spawn(async move { let _ = io::copy(&mut stderr, &mut io::sink()).await; });

    let mut lines = BufReader::new(stdout).lines();
    while let Some(line) = lines.next_line().await? {
        println!("{line}");
    }

    // Call `child.kill().await` from elsewhere to bail out early
    Ok(())
}
```

### Wait for the exit code

Drain both pipes first, then `child.wait().await?` yields the exit
code:

```rust
# use lighty_java::runtime::JavaRuntime;
# use tokio::io::{self, AsyncReadExt};
# use std::path::Path;
# async fn run() -> anyhow::Result<()> {
let rt = JavaRuntime::new("/usr/bin/java".into());
let mut child = rt.execute(vec!["-version".into()], Path::new(".")).await?;
let mut stdout = child.stdout.take().expect("stdout is piped");
let mut stderr = child.stderr.take().expect("stderr is piped");
io::copy(&mut stdout, &mut io::sink()).await?;
io::copy(&mut stderr, &mut io::sink()).await?;

let status = child.wait().await?;
println!("exit code: {:?}", status.code());
# Ok(()) }
```

## Errors

```rust
pub enum JavaRuntimeError {
    NotFound { path: PathBuf },
    NonZeroExit { code: i32 },
    IoCaptureFailure,                    // stdout/stderr couldn't be captured
    Spawn(std::io::Error),
    SignalTerminated,
}
```

## How `lighty-launch` uses it

`lighty-launch::launch::runner` calls `execute` with the full launch
argv (built from `Arguments`) and hands the `Child` to
`handle_console_streams` — every line becomes a `ConsoleOutputEvent`
and the final exit triggers `InstanceExited`. Without the `events`
feature the pipes go to a sink so the JVM never blocks.

## See also

- [`overview.md`](./overview.md) — crate scope
- [`installation.md`](./installation.md) — get a binary path first
- [`../../launch/docs/launch.md`](../../launch/docs/launch.md) — how
  `Launcher` builds the argv
- [`../../launch/docs/arguments.md`](../../launch/docs/arguments.md) —
  JVM arg construction
