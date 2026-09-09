// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Game window detection helpers.

#![cfg(feature = "events")]

use std::time::{Duration, Instant, SystemTime};

use lighty_event::{Event, EventBus, InstanceWindowAppearedEvent};

use crate::instance::INSTANCE_MANAGER;
#[cfg(unix)]
use crate::instance::manager::process_is_running;

/// How often the platform watcher is polled while waiting for the window.
const POLL_INTERVAL: Duration = Duration::from_millis(100);

/// How long to keep polling before giving up on observing the window.
const DETECTION_TIMEOUT: Duration = Duration::from_secs(30);

/// Delay before assuming the window is up, on platforms or sessions that
/// cannot enumerate windows per PID.
const ASSUMED_DELAY: Duration = Duration::from_secs(5);

/// Watches for the game window and emits `InstanceWindowAppeared`, with
/// `detected` telling a real observation from a timed guess. Emits
/// nothing if the process exits first.
pub(crate) async fn detect_window_appearance(
    pid: u32,
    instance_name: String,
    version: String,
    event_bus: EventBus,
) {
    let Some(detected) = watch(pid).await else {
        lighty_core::trace_debug!("[Launch] Window watcher aborted: PID {} exited", pid);
        return;
    };

    if detected {
        lighty_core::trace_info!("[Launch] Window appeared for PID: {}", pid);
    } else {
        lighty_core::trace_info!(
            "[Launch] Assuming window appeared for PID: {} (not observable here)",
            pid
        );
    }

    event_bus.emit(Event::InstanceWindowAppeared(InstanceWindowAppearedEvent {
        pid,
        instance_name,
        version,
        detected,
        timestamp: SystemTime::now(),
    }));
}

/// `Some(true)` once a window owned by `pid` is observed, `Some(false)`
/// when it cannot be observed but the process is still alive, and `None`
/// when the process exited first.
async fn watch(pid: u32) -> Option<bool> {
    let Some(watcher) = platform::watcher() else {
        return assume_after_delay(pid).await;
    };

    let deadline = Instant::now() + DETECTION_TIMEOUT;

    loop {
        if !still_running(pid) {
            return None;
        }

        if watcher.owns_visible_window(pid) {
            return Some(true);
        }

        if Instant::now() >= deadline {
            lighty_core::trace_warn!(
                "[Launch] Window detection timed out for PID: {}, assuming it is up",
                pid
            );
            return still_running(pid).then_some(false);
        }

        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

async fn assume_after_delay(pid: u32) -> Option<bool> {
    tokio::time::sleep(ASSUMED_DELAY).await;
    still_running(pid).then_some(false)
}

/// The registry is the cheap check; the kernel one catches an entry left
/// behind by a console handler that died without unregistering.
fn still_running(pid: u32) -> bool {
    if !INSTANCE_MANAGER.is_alive(pid) {
        return false;
    }

    #[cfg(unix)]
    {
        process_is_running(pid)
    }
    #[cfg(not(unix))]
    {
        true
    }
}

#[cfg(windows)]
mod platform {
    use windows::core::BOOL;
    use windows::Win32::Foundation::{HWND, LPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowThreadProcessId, IsWindowVisible,
    };

    /// Windows always exposes top-level windows with their owning PID.
    pub(super) fn watcher() -> Option<Watcher> {
        Some(Watcher)
    }

    pub(super) struct Watcher;

    impl Watcher {
        /// Returns `true` if `pid` owns at least one visible top-level window.
        pub(super) fn owns_visible_window(&self, pid: u32) -> bool {
            struct EnumData {
                target_pid: u32,
                found: bool,
            }

            unsafe extern "system" fn enum_window_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
                unsafe {
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
    }
}

#[cfg(target_os = "linux")]
mod platform {
    use x11rb::connection::{Connection, RequestConnection};
    use x11rb::protocol::res::{
        query_client_ids, query_version, ClientIdMask, ClientIdSpec, X11_EXTENSION_NAME,
    };
    use x11rb::protocol::xproto::{AtomEnum, ConnectionExt, MapState, Window};
    use x11rb::rust_connection::RustConnection;

    /// `None` when no X server answers, which sends the caller back to
    /// the timed guess. Minecraft goes through GLFW, so X11 or XWayland
    /// is the normal path on Linux.
    pub(super) fn watcher() -> Option<Watcher> {
        let (connection, screen) = x11rb::connect(None).ok()?;
        let root = connection.setup().roots.get(screen)?.root;

        let client_list = intern_atom(&connection, b"_NET_CLIENT_LIST")?;
        let wm_pid = intern_atom(&connection, b"_NET_WM_PID")?;
        let server_side_pid = has_client_ids(&connection);

        Some(Watcher {
            connection,
            root,
            client_list,
            wm_pid,
            server_side_pid,
        })
    }

    /// `QueryClientIds` only exists from X-Resource 1.2 on, and x11rb does
    /// not cache `query_version`, so the answer is settled once here rather
    /// than on every poll.
    fn has_client_ids(connection: &RustConnection) -> bool {
        connection
            .extension_information(X11_EXTENSION_NAME)
            .ok()
            .flatten()
            .is_some()
            && query_version(connection, 1, 2)
                .ok()
                .and_then(|cookie| cookie.reply().ok())
                .is_some_and(|version| (version.server_major, version.server_minor) >= (1, 2))
    }

    fn intern_atom(connection: &RustConnection, name: &[u8]) -> Option<u32> {
        Some(connection.intern_atom(false, name).ok()?.reply().ok()?.atom)
    }

    pub(super) struct Watcher {
        connection: RustConnection,
        root: Window,
        client_list: u32,
        wm_pid: u32,
        server_side_pid: bool,
    }

    impl Watcher {
        /// Returns `true` once a viewable window belongs to `pid`, whether
        /// the application advertised it or the X server had to be asked.
        pub(super) fn owns_visible_window(&self, pid: u32) -> bool {
            self.managed_windows()
                .into_iter()
                .find(|&window| self.window_pid(window) == Some(pid))
                .is_some_and(|window| self.is_viewable(window))
        }

        /// An entry can show up before its window is mapped, so membership
        /// in `_NET_CLIENT_LIST` does not mean the player sees anything yet.
        fn managed_windows(&self) -> Vec<Window> {
            let Ok(cookie) = self.connection.get_property(
                false,
                self.root,
                self.client_list,
                AtomEnum::WINDOW,
                0,
                u32::MAX,
            ) else {
                return Vec::new();
            };

            // `value32` borrows the reply, so it has to outlive the collect.
            let Ok(reply) = cookie.reply() else {
                return Vec::new();
            };

            reply
                .value32()
                .map(|windows| windows.collect())
                .unwrap_or_default()
        }

        /// The property comes first because the application writes it in the
        /// same PID namespace as `Command::spawn`, while the server answers
        /// in its own — they disagree under a sandbox.
        fn window_pid(&self, window: Window) -> Option<u32> {
            self.declared_pid(window)
                .or_else(|| self.server_pid(window))
        }

        fn declared_pid(&self, window: Window) -> Option<u32> {
            let reply = self
                .connection
                .get_property(false, window, self.wm_pid, AtomEnum::CARDINAL, 0, 1)
                .ok()?
                .reply()
                .ok()?;

            reply.value32()?.next()
        }

        /// The only way to reach applications that never advertise a PID,
        /// so LWJGL 2 up to 1.12.2. The null XID is a wildcard matching
        /// every client, so it is refused.
        fn server_pid(&self, window: Window) -> Option<u32> {
            if !self.server_side_pid || window == 0 {
                return None;
            }

            let spec = ClientIdSpec {
                client: window,
                mask: ClientIdMask::LOCAL_CLIENT_PID,
            };
            let reply = query_client_ids(&self.connection, &[spec])
                .ok()?
                .reply()
                .ok()?;

            reply.ids.iter().find_map(|id| id.value.first().copied())
        }

        /// `QueryClientIds` resolves a client slot, not a resource, so a
        /// stale entry answers with whoever inherited the slot. A window
        /// that no longer exists fails here instead.
        fn is_viewable(&self, window: Window) -> bool {
            self.connection
                .get_window_attributes(window)
                .ok()
                .and_then(|cookie| cookie.reply().ok())
                .is_some_and(|attributes| attributes.map_state == MapState::VIEWABLE)
        }
    }
}

#[cfg(not(any(windows, target_os = "linux")))]
mod platform {
    /// macOS could enumerate through `CGWindowListCopyWindowInfo`, which
    /// needs no Screen Recording permission for `kCGWindowOwnerPID`. Not
    /// wired up yet, so this falls back to the timed assumption.
    pub(super) fn watcher() -> Option<Watcher> {
        None
    }

    pub(super) struct Watcher;

    impl Watcher {
        pub(super) fn owns_visible_window(&self, _pid: u32) -> bool {
            false
        }
    }
}
