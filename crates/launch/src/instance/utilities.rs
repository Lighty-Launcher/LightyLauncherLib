use lighty_loaders::types::{InstanceSize, VersionInfo};
use lighty_loaders::types::version_metadata::Version;

use super::errors::{InstanceError, InstanceResult};
use super::INSTANCE_MANAGER;

/// Extension trait providing instance management utilities.
///
/// Auto-implemented for every [`VersionInfo`].
#[allow(async_fn_in_trait)]
pub trait InstanceControl: VersionInfo {
    /// Returns the first PID of a running instance, or `None` if not running.
    fn get_pid(&self) -> Option<u32> {
        INSTANCE_MANAGER.get_pid(self.name())
    }

    /// Returns all PIDs running under this instance name.
    fn get_pids(&self) -> Vec<u32> {
        INSTANCE_MANAGER.get_pids(self.name())
    }

    /// Closes a specific instance by PID.
    async fn close_instance(&self, pid: u32) -> InstanceResult<()> {
        INSTANCE_MANAGER.close_instance(pid).await
    }

    /// Deletes the instance from disk. Errors if any process is still running.
    async fn delete_instance(&self) -> InstanceResult<()> {
        let running_pids = self.get_pids();
        if !running_pids.is_empty() {
            return Err(InstanceError::StillRunning {
                instance_name: self.name().to_string(),
                pids: running_pids,
            });
        }

        tokio::fs::remove_dir_all(self.game_dirs()).await?;

        #[cfg(feature = "events")]
        {
            use lighty_event::{Event, InstanceDeletedEvent, EVENT_BUS};
            use std::time::SystemTime;

            EVENT_BUS.emit(Event::InstanceDeleted(InstanceDeletedEvent {
                instance_name: self.name().to_string(),
                timestamp: SystemTime::now(),
            }));
        }

        lighty_core::trace_info!(instance = %self.name(), "Instance deleted");
        Ok(())
    }

    /// Returns the per-component disk-size breakdown from `version` metadata.
    fn size_of_instance(&self, version: &Version) -> InstanceSize {
        let libraries_size = version.libraries.iter().filter_map(|lib| lib.size).sum();

        let mods_size = version
            .mods
            .as_ref()
            .map(|mods| mods.iter().filter_map(|m| m.size).sum())
            .unwrap_or(0);

        let natives_size = version
            .natives
            .as_ref()
            .map(|natives| natives.iter().filter_map(|n| n.size).sum())
            .unwrap_or(0);

        let client_size = version.client.as_ref().and_then(|c| c.size).unwrap_or(0);

        let assets_size = version
            .assets_index
            .as_ref()
            .and_then(|idx| idx.total_size)
            .unwrap_or(0);

        let total = libraries_size + mods_size + natives_size + client_size + assets_size;

        InstanceSize {
            libraries: libraries_size,
            mods: mods_size,
            natives: natives_size,
            client: client_size,
            assets: assets_size,
            total,
        }
    }
}

impl<T: VersionInfo> InstanceControl for T {}
