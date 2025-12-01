use crate::server::handler::{ClientHandler, HandlerError};
use std::{
    io::{self},
    path::Path,
    process::Command,
};
use tracing::{debug, info, warn};

impl ClientHandler {
    pub(super) fn run(&mut self, path: &Path, brickrun: bool) -> Result<(), HandlerError> {
        debug!("Running the file at ./{}", path.display());

        let (mut reader, writer) = io::pipe()?;

        let arg = format!("./{}", path.display());
        let mut child = if brickrun {
            Command::new("brickrun")
                .arg("-r")
                .arg(arg)
                .stdout(writer.try_clone()?)
                .stderr(writer)
                .spawn()
                .map_err(|e| {
                    warn!("Failed to spawn brickrun command: {e}");
                    e
                })?
        } else {
            Command::new(arg)
                .stdout(writer.try_clone()?)
                .stderr(writer)
                .spawn()
                .map_err(|e| {
                    warn!("Failed to spawn command: {e}");
                    e
                })?
        };

        if let Err(e) = self.transport.forward_output(&mut reader) {
            warn!("Failed to send output to client: {e}");
            child.kill()?;
            return Err(e.into());
        }

        let status = child.wait().map_err(|e| {
            warn!("Failed to wait for exit status of the child: {e}");
            e
        })?;

        if status.success() {
            info!("Child exited with exit status: {status}");
        } else {
            warn!("Child exited with exit status: {status}");
        }

        debug!("Ran file at ./{}", path.display());

        Ok(())
    }
}
