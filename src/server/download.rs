use crate::{
    server::handler::{ClientHandler, HandlerError},
    transport::Transport,
};
use std::{fs::OpenOptions, io::BufWriter, path::Path};
use tracing::{debug, warn};

impl ClientHandler {
    pub(super) fn download(
        &mut self,
        path: &Path,
        use_compression: bool,
    ) -> Result<(), HandlerError> {
        debug!("Downloading file to {:?}", path.display());

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .truncate(true)
            .open(path)
            .inspect_err(|e| {
                warn!(
                    "Failed to open file with create, write and truncate options set to true: {e}"
                )
            })?;

        let mut writer = BufWriter::with_capacity(Transport::FILE_TRANSFER_BUFFER, file);

        self.transport.download_file(&mut writer, use_compression)?;

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
