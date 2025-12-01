use crate::server::handler::{ClientHandler, HandlerError};
use std::{fs::OpenOptions, path::Path};
use tracing::{debug, warn};

impl ClientHandler {
    pub(super) fn upload(&mut self, path: &Path) -> Result<(), HandlerError> {
        debug!("Downloading file to {path:?}");

        let mut new_file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .truncate(true)
            .open(path)
            .map_err(|e| {
                warn!(
                    "Failed to open file with create, write and truncate options set to true: {e}"
                );
                e
            })?;

        self.transport.receive_file(&mut new_file)?;

        Ok(())
    }

    #[cfg(unix)]
    pub(super) fn set_permissions(&mut self, path: &Path) -> Result<(), HandlerError> {
        use std::{
            fs::{File, Permissions},
            os::unix::fs::PermissionsExt,
        };

        let file = File::open(path)?;
        file.set_permissions(Permissions::from_mode(0o755))?;
        debug!("Set file permissions to execute on unix-systems");

        Ok(())
    }
}
