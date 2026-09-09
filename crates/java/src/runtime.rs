// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Java process execution wrapper.

use crate::errors::{JavaRuntimeError, JavaRuntimeResult};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::{Child, Command};

/// Wrapper around a Java binary path for process execution
pub struct JavaRuntime(pub PathBuf);

impl JavaRuntime {
    /// Creates a new JavaRuntime from a binary path
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }

    /// Spawns a Java process with the given arguments in `game_dir`.
    pub async fn execute(&self, arguments: Vec<String>, game_dir: &Path) -> JavaRuntimeResult<Child> {
        if !self.0.exists() {
            return Err(JavaRuntimeError::NotFound {
                path: self.0.clone(),
            });
        }

        lighty_core::trace_debug!("Spawning Java process: {:?}", &self.0);

        let mut command = Command::new(&self.0);
        command
            .current_dir(game_dir)
            .args(arguments)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // On Windows, hide the console window
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            command.creation_flags(CREATE_NO_WINDOW);
        }

        let child = command.spawn()?;

        lighty_core::trace_info!("Java process spawned successfully");
        Ok(child)
    }
}
