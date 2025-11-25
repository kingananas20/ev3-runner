use crate::server::handler::{ClientHandler, HandlerError};
use std::{
    fs::{OpenOptions, Permissions},
    os::unix::fs::PermissionsExt,
    path::Path,
};
use tracing::{debug, trace, warn};

impl ClientHandler {
    pub(super) fn upload(&mut self, path: &Path, size: u64) -> Result<(), HandlerError> {
        debug!("Downloading file to {path:?} with the size of {size} bytes");

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .map_err(|e| {
                warn!(
                    "Failed to open file with create, write and truncate options set to true: {e}"
                );
                e
            })?;

        self.transport.receive_file(&mut file, size)?;

        if cfg!(target_os = "linux") {
            file.set_permissions(Permissions::from_mode(0o755))?;
            trace!("Set file permissions on linux");
        }

        Ok(())
    }
}
