use crate::VERSION;
use crate::protocol::{VersionHeader, VersionResponse, VersionStatus};
use crate::server::handler::{ClientHandler, HandlerError};
use tracing::{debug, warn};

impl ClientHandler {
    pub(super) fn check_version(&mut self) -> Result<(), HandlerError> {
        let version_header = self.transport.read_and_decode::<VersionHeader>()?;

        let mut version_response = VersionResponse(VersionStatus::Match);
        if version_header.0 != VERSION {
            version_response = VersionResponse(VersionStatus::Mismatch(VERSION.to_owned()));
            self.transport.encode_and_write(version_response)?;
            warn!(
                "Client version ({}) does not match server version ({VERSION})",
                version_header.0
            );
            return Err(HandlerError::VersionMismatch(VERSION.to_owned()));
        } else {
            self.transport.encode_and_write(version_response)?;
            debug!("No version mismatch");
        };

        Ok(())
    }
}
