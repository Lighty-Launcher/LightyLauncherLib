// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Java process execution wrapper.

use crate::errors::{JavaRuntimeError, JavaRuntimeResult};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::AsyncReadExt;
use tokio::process::{Child, Command};
use tokio::sync::oneshot::Receiver;

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

    /// Streams stdout/stderr from the process with custom handlers.
    ///
    /// Calls `on_stdout`/`on_stderr` callbacks until the process exits or
    /// `terminator` fires. Exit code -1073740791 (Windows forceful termination)
    /// is not treated as an error.
    pub async fn handle_io<D: Send + Sync>(
        &self,
        process: &mut Child,
        on_stdout: fn(&D, &[u8]) -> JavaRuntimeResult<()>,
        on_stderr: fn(&D, &[u8]) -> JavaRuntimeResult<()>,
        terminator: Receiver<()>,
        data: &D,
    ) -> JavaRuntimeResult<()> {
        let mut stdout = process
            .stdout
            .take()
            .ok_or(JavaRuntimeError::IoCaptureFailure)?;
        let mut stderr = process
            .stderr
            .take()
            .ok_or(JavaRuntimeError::IoCaptureFailure)?;

        // 8KB is optimal for most Java logs while avoiding stack overflow
        let mut stdout_buffer = [0u8; 8192];
        let mut stderr_buffer = [0u8; 8192];

        tokio::pin!(terminator);

        loop {
            tokio::select! {
                result = stdout.read(&mut stdout_buffer) => {
                    match result {
                        Ok(bytes_read) if bytes_read > 0 => {
                            let _ = on_stdout(data, &stdout_buffer[..bytes_read]);
                        }
                        Ok(_) => {}, // EOF reached
                        Err(_) => break, // Stream closed
                    }
                },

                result = stderr.read(&mut stderr_buffer) => {
                    match result {
                        Ok(bytes_read) if bytes_read > 0 => {
                            let _ = on_stderr(data, &stderr_buffer[..bytes_read]);
                        }
                        Ok(_) => {}, // EOF reached
                        Err(_) => break, // Stream closed
                    }
                },

                _ = &mut terminator => {
                    lighty_core::trace_debug!("Termination signal received, killing process");
                    process.kill().await?;
                    break;
                },

                exit_result = process.wait() => {
                    let exit_status = exit_result?;
                    let exit_code = exit_status.code().unwrap_or(7900);

                    lighty_core::trace_debug!("Java process exited with code: {}", exit_code);

                    // -1073740791 = Windows forceful termination (not an error)
                    if exit_code != 0 && exit_code != -1073740791 {
                        return Err(JavaRuntimeError::NonZeroExit { code: exit_code });
                    }

                    break;
                },
            }
        }

        Ok(())
    }
}
