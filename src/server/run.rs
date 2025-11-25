use crate::server::handler::ClientHandler;
use std::{
    io::{self, Error},
    path::Path,
    process::Command,
};
use tracing::{debug, info, warn};

impl ClientHandler {
    pub(super) fn run(&mut self, path: &Path) -> Result<(), Error> {
        debug!("Running the file at ./{}", path.display());

        let (mut reader, writer) = io::pipe()?;

        let mut child = Command::new(format!("./{}", path.display()))
            .stdout(writer.try_clone()?)
            .stderr(writer)
            .spawn()
            .map_err(|e| {
                warn!("Failed to spawn command: {e}");
                e
            })?;

        if let Err(e) = self.send_output(&mut reader) {
            warn!("Failed to send output to client: {e}");
            child.kill()?;
            return Err(e);
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

        debug!("Ran file at {path:?}");

        Ok(())
    }
}
