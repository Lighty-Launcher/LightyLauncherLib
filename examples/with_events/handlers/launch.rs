//! Install + launch events.
//!
//! `InstallStarted` / `InstallProgress` / `InstallCompleted` share
//! state (running totals) so the on-screen percentage is meaningful.
//! [`InstallProgress`] owns that state and is threaded back into
//! [`log`] by the dispatcher.

use lighty_launcher::prelude::*;

use super::flush_stdout;

/// Per-launch download counters kept across `InstallStarted` /
/// `InstallProgress` events.
#[derive(Default)]
pub struct InstallProgress {
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
}

pub fn log(event: LaunchEvent, progress: &mut InstallProgress) {
    match event {
        LaunchEvent::IsInstalled { version } => {
            trace_info!("{} is already installed and up-to-date!", version);
        }
        LaunchEvent::InstallStarted { total_bytes, .. } => {
            progress.total_bytes = total_bytes;
            trace_info!("Installing: {} MB total", total_bytes / 1_000_000);
        }
        LaunchEvent::InstallProgress { bytes } => {
            progress.downloaded_bytes += bytes;
            let percent =
                (progress.downloaded_bytes as f64 / progress.total_bytes as f64) * 100.0;
            print!("\rProgress: {:.1}%", percent);
            flush_stdout();
        }
        LaunchEvent::InstallCompleted { .. } => {
            trace_info!("\nInstallation completed!");
        }
        _ => {}
    }
}
