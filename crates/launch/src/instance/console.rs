#[cfg(feature = "events")]
use std::io::ErrorKind;
#[cfg(feature = "events")]
use std::time::SystemTime;

#[cfg(not(feature = "events"))]
use tokio::io;
#[cfg(feature = "events")]
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::process::Child;

#[cfg(feature = "events")]
use lighty_event::{ConsoleOutputEvent, ConsoleStream, Event, EventBus, InstanceExitedEvent};

/// Spawns tasks that stream stdout/stderr from the child, emit console
/// events, and unregister the instance when the process exits.
pub(crate) async fn handle_console_streams(
    pid: u32,
    instance_name: String,
    mut child: Child,
    #[cfg(feature = "events")] event_bus: Option<EventBus>,
) {
    // The JVM blocks on `write` once the pipe buffer fills, so both pipes
    // must be drained for the whole life of the process.
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    #[cfg(feature = "events")]
    {
        if let Some(stdout) = stdout {
            tokio::spawn(emit_lines(
                stdout,
                pid,
                instance_name.clone(),
                ConsoleStream::Stdout,
                event_bus.clone(),
            ));
        }

        if let Some(stderr) = stderr {
            tokio::spawn(emit_lines(
                stderr,
                pid,
                instance_name.clone(),
                ConsoleStream::Stderr,
                event_bus.clone(),
            ));
        }
    }

    #[cfg(not(feature = "events"))]
    {
        if let Some(mut stdout) = stdout {
            tokio::spawn(async move {
                let _ = io::copy(&mut stdout, &mut io::sink()).await;
            });
        }

        if let Some(mut stderr) = stderr {
            tokio::spawn(async move {
                let _ = io::copy(&mut stderr, &mut io::sink()).await;
            });
        }
    }

    let exit = child.wait().await;

    // Unregister before announcing the exit: a watcher gating on
    // `is_alive` must not still see the PID alive once `InstanceExited`
    // is out, or it could emit `InstanceWindowAppeared` after the fact.
    use super::INSTANCE_MANAGER;
    INSTANCE_MANAGER.unregister_instance(pid).await;

    match exit {
        Ok(status) => {
            #[cfg(feature = "events")]
            if let Some(ref bus) = event_bus {
                bus.emit(Event::InstanceExited(InstanceExitedEvent {
                    pid,
                    instance_name: instance_name.clone(),
                    exit_code: status.code(),
                    timestamp: SystemTime::now(),
                }));
            }

            lighty_core::trace_info!(
                pid = pid,
                instance = %instance_name,
                exit_code = ?status.code(),
                "Instance exited"
            );
        }
        Err(e) => {
            lighty_core::trace_error!(
                pid = pid,
                instance = %instance_name,
                error = %e,
                "Error waiting for instance"
            );
        }
    }
}

/// Emits one console event per line until the stream ends.
#[cfg(feature = "events")]
async fn emit_lines<R>(
    stream: R,
    pid: u32,
    instance_name: String,
    console_stream: ConsoleStream,
    event_bus: Option<EventBus>,
) where
    R: AsyncRead + Unpin + Send + 'static,
{
    let mut lines = BufReader::new(stream).lines();

    loop {
        match lines.next_line().await {
            Ok(Some(line)) => {
                if let Some(ref bus) = event_bus {
                    bus.emit(Event::ConsoleOutput(ConsoleOutputEvent {
                        pid,
                        instance_name: instance_name.clone(),
                        stream: console_stream,
                        line,
                        timestamp: SystemTime::now(),
                    }));
                }
            }
            Ok(None) => break,
            // Ending the loop would close the pipe and cut the console for good.
            Err(err) if err.kind() == ErrorKind::InvalidData => continue,
            Err(_) => break,
        }
    }
}
