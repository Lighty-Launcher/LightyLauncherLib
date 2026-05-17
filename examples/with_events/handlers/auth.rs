//! Authentication events.

use lighty_launcher::prelude::*;

pub fn log(event: AuthEvent) {
    match event {
        AuthEvent::AuthenticationStarted { provider } => {
            trace_info!("Authenticating with {}...", provider);
        }
        AuthEvent::AuthenticationSuccess { username, .. } => {
            trace_info!("Authenticated as {}", username);
        }
        AuthEvent::AuthenticationFailed { error, .. } => {
            trace_error!("Authentication failed: {}", error);
        }
        _ => {}
    }
}
