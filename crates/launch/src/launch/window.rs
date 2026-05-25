// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Game window detection helpers.

#![cfg(feature = "events")]

use lighty_event::{Event, EventBus, InstanceWindowAppearedEvent};

use crate::instance::INSTANCE_MANAGER;

/// Watches for the game window to appear and emits `InstanceWindowAppeared`.
///
/// On Windows: polls every 100ms for up to 30s using `EnumWindows`.
/// On other platforms: emits unconditionally after a 5s delay (heuristic,
/// since there is no portable per-PID window enumeration).
///
/// Bails out if the PID exits before detection completes, so a stale
/// event doesn't override `InstanceExited` downstream.
pub(crate) async fn detect_window_appearance(
    pid: u32,
    instance_name: String,
    version: String,
    event_bus: EventBus,
) {
    #[cfg(windows)]
    {
        use std::time::Duration;

        let max_attempts = 300;
        let check_interval = Duration::from_millis(100);

        for _ in 0..max_attempts {
            if !INSTANCE_MANAGER.is_alive(pid) {
                lighty_core::trace_debug!(
                    "[Launch] Window watcher aborted: PID {} exited",
                    pid
                );
                return;
            }

            if has_visible_window(pid) {
                lighty_core::trace_info!("[Launch] Window appeared for PID: {}", pid);

                event_bus.emit(Event::InstanceWindowAppeared(InstanceWindowAppearedEvent {
                    pid,
                    instance_name,
                    version,
                    timestamp: std::time::SystemTime::now(),
                }));
                return;
            }

            tokio::time::sleep(check_interval).await;
        }

        lighty_core::trace_warn!("[Launch] Window detection timed out for PID: {}", pid);
    }

    #[cfg(not(windows))]
    {
        // No portable per-PID window enumeration: best-effort 5s delay.
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;

        if !INSTANCE_MANAGER.is_alive(pid) {
            lighty_core::trace_debug!(
                "[Launch] Window watcher aborted: PID {} exited",
                pid
            );
            return;
        }

        lighty_core::trace_info!(
            "[Launch] Assuming window appeared for PID: {} (non-Windows platform)",
            pid
        );

        event_bus.emit(Event::InstanceWindowAppeared(InstanceWindowAppearedEvent {
            pid,
            instance_name,
            version,
            timestamp: std::time::SystemTime::now(),
        }));
    }
}

/// Returns `true` if the given PID owns at least one visible top-level window.
///
/// Windows-only; uses `EnumWindows` + `IsWindowVisible` + `GetWindowThreadProcessId`.
#[cfg(windows)]
fn has_visible_window(pid: u32) -> bool {
    use windows::core::BOOL;
    use windows::Win32::Foundation::{HWND, LPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowThreadProcessId, IsWindowVisible,
    };

    struct EnumData {
        target_pid: u32,
        found: bool,
    }

    unsafe extern "system" fn enum_window_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let data = &mut *(lparam.0 as *mut EnumData);

        if IsWindowVisible(hwnd).as_bool() {
            let mut window_pid: u32 = 0;
            GetWindowThreadProcessId(hwnd, Some(&mut window_pid));

            if window_pid == data.target_pid {
                data.found = true;
                return BOOL(0); // Stop enumeration
            }
        }

        BOOL(1) // Continue enumeration
    }

    let mut data = EnumData {
        target_pid: pid,
        found: false,
    };

    unsafe {
        let _ = EnumWindows(
            Some(enum_window_callback),
            LPARAM(&mut data as *mut _ as isize),
        );
    }

    data.found
}
