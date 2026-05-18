# Runtime — spawn + stream a `java` process

`JavaRuntime` is a thin wrapper around `tokio::process::Command`. It
spawns the binary, returns the `Child`, and offers `handle_io` to
stream stdout / stderr until the process exits (or a one-shot
terminator fires).

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

    pub async fn handle_io<D: Send + Sync>(
        &self,
        process:    &mut tokio::process::Child,
        on_stdout:  fn(&D, &[u8]) -> JavaRuntimeResult<()>,
        on_stderr:  fn(&D, &[u8]) -> JavaRuntimeResult<()>,
        terminator: tokio::sync::oneshot::Receiver<()>,
        data:       &D,
    ) -> JavaRuntimeResult<()>;
}
```

`game_dir` becomes the process's working directory (this is what gets
passed to Minecraft via `${game_directory}`).

`handle_io` takes **function pointers** (not closures) so they can
cross the Tokio `select!` cleanly. The `data` argument is an opaque
context bag — pass whatever the callbacks need to share (a logger
handle, an `EventBus`, …).

Windows-only detail: `execute` sets `CREATE_NO_WINDOW` so spawned
processes don't pop up a console window.

## Examples

### Print `java -version`

```rust
use lighty_java::runtime::JavaRuntime;
use tokio::sync::oneshot;
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let rt = JavaRuntime::new("/usr/bin/java".into());
    let mut child = rt.execute(vec!["-version".into()], Path::new(".")).await?;

    let (_tx, rx) = oneshot::channel();
    rt.handle_io::<()>(
        &mut child,
        |_, b| { print!("{}",  String::from_utf8_lossy(b)); Ok(()) },
        |_, b| { eprint!("{}", String::from_utf8_lossy(b)); Ok(()) },
        rx,
        &(),
    ).await?;
    Ok(())
}
```

`java -version` writes to stderr — both callbacks get hit.

### Launch a JAR with memory tuning

```rust
use lighty_java::runtime::JavaRuntime;
use tokio::sync::oneshot;
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

    let (term_tx, term_rx) = oneshot::channel();
    let io = rt.handle_io::<()>(
        &mut child,
        |_, b| { print!("{}",  String::from_utf8_lossy(b)); Ok(()) },
        |_, b| { eprint!("{}", String::from_utf8_lossy(b)); Ok(()) },
        term_rx,
        &(),
    );

    // Fire `term_tx` from elsewhere to bail out early
    let _ = term_tx;
    io.await?;
    Ok(())
}
```

### Wait for the exit code

`handle_io` returns once the process exits or the terminator fires.
The exit code is available via `child.wait().await?`:

```rust
# use lighty_java::runtime::JavaRuntime;
# use tokio::sync::oneshot;
# use std::path::Path;
# async fn run() -> anyhow::Result<()> {
let rt = JavaRuntime::new("/usr/bin/java".into());
let mut child = rt.execute(vec!["-version".into()], Path::new(".")).await?;
let (_tx, rx) = oneshot::channel();
rt.handle_io::<()>(&mut child,
    |_, _| Ok(()), |_, _| Ok(()), rx, &()).await?;

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

The Windows forceful-termination code `-1073740791` (0xC0000409) is
treated as a normal exit, not an error.

## How `lighty-launch` uses it

`lighty-launch::launcher::Launcher` calls `execute` with the full
launch argv (built from `Arguments`) and pipes `handle_io` into the
event bus — every line becomes a `ConsoleOutputEvent` and the final
exit triggers `InstanceExited`. The same `oneshot` terminator is
hooked to the cancel button in the host UI.

## See also

- [`overview.md`](./overview.md) — crate scope
- [`installation.md`](./installation.md) — get a binary path first
- [`../../launch/docs/launch.md`](../../launch/docs/launch.md) — how
  `Launcher` builds the argv
- [`../../launch/docs/arguments.md`](../../launch/docs/arguments.md) —
  JVM arg construction
