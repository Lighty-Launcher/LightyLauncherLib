//! Instance lifecycle events (launched / exited / deleted).

use lighty_launcher::prelude::*;

pub fn log_launched(e: InstanceLaunchedEvent) {
    trace_info!("\n[EVENT] Instance '{}' launched", e.instance_name);
    trace_info!("PID: {}", e.pid);
    trace_info!("Version: {}", e.version);
    trace_info!("Player: {}", e.username);
}

pub fn log_exited(e: InstanceExitedEvent) {
    trace_info!(
        "\n[EVENT] Instance '{}' exited with code: {:?}",
        e.instance_name, e.exit_code
    );
}

pub fn log_deleted(e: InstanceDeletedEvent) {
    trace_info!("\n[EVENT] Instance '{}' deleted", e.instance_name);
}
