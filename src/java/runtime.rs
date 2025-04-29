use std::env::args;
use anyhow::{bail, Result};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use log::info;
use tokio::io::AsyncReadExt;
use tokio::process::{Child, Command};
use tokio::sync::oneshot::Receiver;
use tracing::{debug, info_span};
pub struct JavaRuntime(pub PathBuf);

impl JavaRuntime {
    pub fn new(path: PathBuf) -> JavaRuntime {
        JavaRuntime(path)
    }

    pub async fn execute(&self, arguments: Vec<String>, game_dir: &Path) -> Result<Child> {
        if !self.0.exists() {
            bail!("Java runtime not found at: {}", self.0.display());
        }
        let mut command = Command::new(&self.0);


        //DEBUG TEST
        debug!("Executing Java runtime: {}", self.0.display());
        info!("Executing Java runtime: {}", self.0.display());
        let mut test = arguments.clone();
        info!("Arguments: {:?}", command.args(test));




        command.current_dir(game_dir);

        command.args(arguments);
        println!("Java runtime: {}", self.0.display());

        command.stderr(Stdio::piped()).stdout(Stdio::piped());

        let child = command.spawn()?;
        Ok(child)
    }

    pub async fn handle_io<D: Send + Sync>(
        &self,
        running_task: &mut Child,
        on_stdout: fn(&D, &[u8]) -> Result<()>,
        on_stderr: fn(&D, &[u8]) -> Result<()>,
        terminator: Receiver<()>,
        data: &D,
    ) -> Result<()> {
        let mut stdout = running_task.stdout.take().unwrap();
        let mut stderr = running_task.stderr.take().unwrap();

        let mut stdout_buf = vec![0; 1024];
        let mut stderr_buf = vec![0; 1024];

        tokio::pin!(terminator);

        loop {
            tokio::select! {
                read_len = stdout.read(&mut stdout_buf) => {
                    let _ = on_stdout(&data, &stdout_buf[..read_len?]);
                },
                read_len = stderr.read(&mut stderr_buf) => {
                    let _ = on_stderr(&data, &stderr_buf[..read_len?]);
                },
                _ = &mut terminator => {
                    running_task.kill().await?;
                    break;
                },
                exit_status = running_task.wait() => {
                    let code = exit_status?.code().unwrap_or(7900); // 7900 = unwrap failed error code

                    debug!("Process exited with code: {}", code);
                    if code != 0 && code != -1073740791 { // -1073740791 = happens when the process is killed forcefully, we don't want to bail in this case
                        bail!("Process exited with non-zero exit code: {}.", code);
                    }
                    break;
                },
            }
        }
        Ok(())
    }
}