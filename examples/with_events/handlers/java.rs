//! JRE lifecycle events (Adoptium/Zulu/etc. download + extraction).

use lighty_launcher::prelude::*;

use super::flush_stdout;

pub fn log(event: JavaEvent) {
    match event {
        JavaEvent::JavaNotFound { distribution, version } => {
            trace_info!("[Java] {} {} not found, downloading...", distribution, version);
        }
        JavaEvent::JavaAlreadyInstalled { distribution, version, .. } => {
            trace_info!("[Java] {} {} already installed", distribution, version);
        }
        JavaEvent::JavaDownloadStarted { distribution, version, total_bytes } => {
            trace_info!(
                "[Java] Downloading {} {} ({} MB)",
                distribution, version, total_bytes / 1_000_000
            );
        }
        JavaEvent::JavaDownloadProgress { bytes } => {
            print!("\r[Java] Download progress: {} MB", bytes / 1_000_000);
            flush_stdout();
        }
        JavaEvent::JavaDownloadCompleted { distribution, version } => {
            trace_info!("\n[Java] {} {} download completed", distribution, version);
        }
        JavaEvent::JavaExtractionStarted { distribution, version } => {
            trace_info!("[Java] Extracting {} {}...", distribution, version);
        }
        JavaEvent::JavaExtractionCompleted { distribution, version, .. } => {
            trace_info!("[Java] {} {} extraction completed", distribution, version);
        }
        _ => {}
    }
}
