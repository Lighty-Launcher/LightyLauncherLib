#[cfg(feature = "events")]
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;

#[cfg(feature = "events")]
use lighty_event::EventBus;

/// Spawns tasks that stream stdout/stderr from the child, emit console
/// events, and unregister the instance when the process exits.
pub(crate) async fn handle_console_streams(
    pid: u32,
    instance_name: String,
    mut child: Child,
    #[cfg(feature = "events")] event_bus: Option<EventBus>,
) {
    // Without the events feature there is no consumer for stdout/stderr,
    // so don't bother spawning the reader tasks.
    #[cfg(feature = "events")]
    {
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        if let Some(stdout) = stdout {
            let instance_name = instance_name.clone();
            let event_bus_clone = event_bus.clone();

            tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();

                while let Ok(Some(line)) = lines.next_line().await {
                    use lighty_event::{ConsoleOutputEvent, ConsoleStream, Event};
                    use std::time::SystemTime;

                    if let Some(ref bus) = event_bus_clone {
                        bus.emit(Event::ConsoleOutput(ConsoleOutputEvent {
                            pid,
                            instance_name: instance_name.clone(),
                            stream: ConsoleStream::Stdout,
                            line,
                            timestamp: SystemTime::now(),
                        }));
                    }
                }
            });
        }

        if let Some(stderr) = stderr {
            let instance_name = instance_name.clone();
            let event_bus_clone = event_bus.clone();

            tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();

                while let Ok(Some(line)) = lines.next_line().await {
                    use lighty_event::{ConsoleOutputEvent, ConsoleStream, Event};
                    use std::time::SystemTime;

                    if let Some(ref bus) = event_bus_clone {
                        bus.emit(Event::ConsoleOutput(ConsoleOutputEvent {
                            pid,
                            instance_name: instance_name.clone(),
                            stream: ConsoleStream::Stderr,
                            line,
                            timestamp: SystemTime::now(),
                        }));
                    }
                }
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
            {
                use lighty_event::{Event, InstanceExitedEvent};
                use std::time::SystemTime;

                if let Some(ref bus) = event_bus {
                    bus.emit(Event::InstanceExited(InstanceExitedEvent {
                        pid,
                        instance_name: instance_name.clone(),
                        exit_code: status.code(),
                        timestamp: SystemTime::now(),
                    }));
                }
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
