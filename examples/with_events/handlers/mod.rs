//! Event logger split by event domain. Each submodule exposes a
//! single `log(...)` function focused on its domain's payloads.
//!
//! Wire-up:
//! - [`spawn_logger`] subscribes to the bus and runs [`dispatch`] in a
//!   detached task.
//! - [`dispatch`] is the one-pass top-level match — every arm just
//!   delegates to the right submodule.

mod auth;
mod console;
mod core;
mod instance;
mod java;
mod launch;
mod loader;

pub use launch::InstallProgress;

use lighty_launcher::prelude::*;

/// Subscribes to the bus and spawns a detached task that logs every
/// event in real time.
pub fn spawn_logger(event_bus: &EventBus) {
    let mut receiver = event_bus.subscribe();
    tokio::spawn(async move {
        let mut progress = InstallProgress::default();
        while let Ok(event) = receiver.next().await {
            dispatch(event, &mut progress);
        }
    });
}

fn dispatch(event: Event, progress: &mut InstallProgress) {
    match event {
        Event::Auth(e)             => auth::log(e),
        Event::Launch(e)           => launch::log(e, progress),
        Event::Java(e)             => java::log(e),
        Event::Loader(e)           => loader::log(e),
        Event::Core(e)             => core::log(e),
        Event::InstanceLaunched(e) => instance::log_launched(e),
        Event::InstanceExited(e)   => instance::log_exited(e),
        Event::InstanceDeleted(e)  => instance::log_deleted(e),
        Event::ConsoleOutput(e)    => console::log(e),
        _ => {}
    }
}

/// Shared helper — flushes stdout so in-place progress lines render
/// without buffering.
pub(super) fn flush_stdout() {
    std::io::Write::flush(&mut std::io::stdout()).ok();
}
