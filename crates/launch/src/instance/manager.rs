use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{PoisonError, RwLock};
use std::time::SystemTime;

use super::errors::{InstanceError, InstanceResult};

/// Internal representation of a running game instance.
pub(crate) struct GameInstance {
    pub pid: u32,
    pub instance_name: String,
    #[allow(dead_code)]
    pub version: String,
    #[allow(dead_code)]
    pub username: String,
    #[allow(dead_code)]
    pub game_dir: PathBuf,
    #[allow(dead_code)]
    pub started_at: SystemTime,
}

/// Internal manager for tracking running game instances
pub(crate) struct InstanceManager {
    instances: RwLock<HashMap<u32, GameInstance>>,
}

/// Global instance manager
pub(crate) static INSTANCE_MANAGER: Lazy<InstanceManager> = Lazy::new(InstanceManager::new);

impl InstanceManager {
    /// Create a new instance manager
    pub fn new() -> Self {
        Self {
            instances: RwLock::new(HashMap::new()),
        }
    }

    /// Get the first PID for a given instance name
    pub fn get_pid(&self, instance_name: &str) -> Option<u32> {
        let instances = self
            .instances
            .read()
            .unwrap_or_else(PoisonError::into_inner);
        instances
            .values()
            .find(|inst| inst.instance_name == instance_name)
            .map(|inst| inst.pid)
    }

    /// Get all PIDs for a given instance name
    pub fn get_pids(&self, instance_name: &str) -> Vec<u32> {
        let instances = self
            .instances
            .read()
            .unwrap_or_else(PoisonError::into_inner);
        instances
            .values()
            .filter(|inst| inst.instance_name == instance_name)
            .map(|inst| inst.pid)
            .collect()
    }

    /// Returns `true` if `pid` is still tracked as a running instance.
    /// Reads the registry, not the OS: it goes stale if the console
    /// handler dies without unregistering — see [`process_is_running`].
    pub fn is_alive(&self, pid: u32) -> bool {
        let instances = self
            .instances
            .read()
            .unwrap_or_else(PoisonError::into_inner);
        instances.contains_key(&pid)
    }

    /// Register a new running instance.
    ///
    /// Returns `Err(InstanceError::DuplicatePid)` if `instance.pid` is
    /// already tracked (race between two concurrent `register_instance`
    /// calls, or OS PID reuse before our `unregister_instance` fired).
    pub async fn register_instance(&self, instance: GameInstance) -> InstanceResult<()> {
        let mut instances = self
            .instances
            .write()
            .unwrap_or_else(PoisonError::into_inner);
        if let Some(existing) = instances.get(&instance.pid) {
            return Err(InstanceError::DuplicatePid {
                pid: instance.pid,
                existing_instance: existing.instance_name.clone(),
            });
        }
        instances.insert(instance.pid, instance);
        Ok(())
    }

    /// Unregister an instance by PID
    pub async fn unregister_instance(&self, pid: u32) {
        let mut instances = self
            .instances
            .write()
            .unwrap_or_else(PoisonError::into_inner);
        instances.remove(&pid);
    }

    /// Close an instance by PID
    ///
    /// Kills the process using the system's kill mechanism.
    /// The instance will be unregistered automatically by the console handler.
    pub async fn close_instance(&self, pid: u32) -> InstanceResult<()> {
        if !self
            .instances
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .contains_key(&pid)
        {
            return Err(InstanceError::NotFound { pid });
        }

        // Unix uses SIGTERM so the JVM runs its shutdown hooks (avoids losing
        // unflushed world state); Windows shells out to `taskkill /F` to
        // terminate the process tree.
        #[cfg(target_os = "windows")]
        {
            use std::process::Command;
            let output = Command::new("taskkill")
                .args(&["/PID", &pid.to_string(), "/F"])
                .output()?;

            if !output.status.success() {
                return Err(InstanceError::KillFailed {
                    pid,
                    reason: String::from_utf8_lossy(&output.stderr).trim().to_string(),
                });
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;

            kill(Pid::from_raw(pid as i32), Signal::SIGTERM).map_err(|err| {
                InstanceError::KillFailed {
                    pid,
                    reason: err.to_string(),
                }
            })?;
        }

        // Only now: a failed kill must leave the instance registered, or the
        // process would keep running with nothing tracking it.
        self.instances
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(&pid);

        lighty_core::trace_info!(pid = pid, "Instance killed");
        Ok(())
    }
}

/// Asks the kernel whether `pid` still exists. Signal 0 runs the check
/// without delivering anything; a zombie counts, which is fine since the
/// console handler unregisters as soon as it reaps the child.
#[cfg(unix)]
pub(crate) fn process_is_running(pid: u32) -> bool {
    use nix::errno::Errno;
    use nix::sys::signal::kill;
    use nix::unistd::Pid;

    !matches!(kill(Pid::from_raw(pid as i32), None), Err(Errno::ESRCH))
}
