use crate::client::clientsession::{ClientError, ClientSession};
use crate::protocol::{MatchStatus, PathStatus, Request, Validation};
use std::fs::File;
use std::io::BufReader;
use tracing::{error, info};

impl ClientSession {
    pub(super) fn validation(
        &mut self,
        req: Request,
        mut reader: BufReader<File>,
    ) -> Result<(), ClientError> {
        let validation = self.transport.read_and_decode::<Validation>()?;

        if validation.password == MatchStatus::Mismatch {
            error!("Wrong password");
            return Err(ClientError::PasswordNotValid);
        }
        info!("Correct password");

        if validation.path != PathStatus::Valid {
            error!("Remote path is not valid: {}", validation.path);
            return Err(validation.path.into());
        }
        info!("Remota path is valid");

        if validation.hash == MatchStatus::Mismatch {
            info!("Uploading file because remote hash did not match");
            self.transport.send_file(&mut reader, req.size)?;
        } else {
            info!("Remote file already up to date, no upload needed");
        }

        Ok(())
    }
}
