//! Low-level core events — archive extraction progress (zips, tarballs).

use lighty_launcher::prelude::*;

use super::flush_stdout;

pub fn log(event: CoreEvent) {
    match event {
        CoreEvent::ExtractionStarted { archive_type, file_count, .. } => {
            if file_count > 0 {
                trace_info!(
                    "[Core] Extracting {} archive ({} files)...",
                    archive_type, file_count
                );
            } else {
                trace_info!("[Core] Extracting {} archive...", archive_type);
            }
        }
        CoreEvent::ExtractionProgress { files_extracted, total_files } => {
            if total_files > 0 {
                let percent = (files_extracted as f64 / total_files as f64) * 100.0;
                print!(
                    "\r[Core] Extraction progress: {}/{} files ({:.1}%)",
                    files_extracted, total_files, percent
                );
            } else {
                print!("\r[Core] Extraction progress: {} files", files_extracted);
            }
            flush_stdout();
        }
        CoreEvent::ExtractionCompleted { archive_type, files_extracted } => {
            trace_info!(
                "\n[Core] {} extraction completed ({} files)",
                archive_type, files_extracted
            );
        }
    }
}
