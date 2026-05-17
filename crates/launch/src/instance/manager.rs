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
        let mut instances = self
            .instances
            .write()
            .unwrap_or_else(PoisonError::into_inner);
        instances
            .remove(&pid)
            .ok_or(InstanceError::NotFound { pid })?;

        drop(instances);

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
                lighty_core::trace_warn!(pid = pid, "Failed to kill process");
            } else {
                lighty_core::trace_info!(pid = pid, "Instance killed");
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;

            match kill(Pid::from_raw(pid as i32), Signal::SIGTERM) {
                Ok(_) => {
                    lighty_core::trace_info!(pid = pid, "Instance killed");
                }
                Err(e) => {
                    lighty_core::trace_warn!(pid = pid, error = %e, "Failed to kill process");
                    return Err(InstanceError::Io(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to kill process: {}", e),
                    )));
                }
            }
        }

        Ok(())
    }
}
