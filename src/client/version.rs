use crate::VERSION;
use crate::client::clientsession::{ClientError, ClientSession};
use crate::protocol::{VersionHeader, VersionResponse, VersionStatus};
use tracing::{error, info};

impl ClientSession {
    pub(super) fn check_version(&mut self) -> Result<(), ClientError> {
        self.transport
            .encode_and_write(VersionHeader(VERSION.to_owned()))?;

        let version_response = self.transport.read_and_decode::<VersionResponse>()?;
        if let VersionResponse(VersionStatus::Mismatch(server_version)) = version_response {
            error!("Server version ({server_version}) does not match client version ({VERSION})");
            return Err(ClientError::VersionMismatch(server_version));
        };

        info!("No version mismatch");

        Ok(())
    }
}
