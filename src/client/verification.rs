use crate::client::clientsession::{ClientError, ClientSession};
use crate::protocol::{MatchStatus, Request, Verification};
use std::fs::File;
use std::io::BufReader;
use tracing::{error, info};

impl ClientSession {
    pub(super) fn verification(
        &mut self,
        req: Request,
        mut reader: BufReader<File>,
    ) -> Result<(), ClientError> {
        let verification = self.transport.read_and_decode::<Verification>()?;

        if verification.password == MatchStatus::Mismatch {
            error!("Wrong password");
            return Err(ClientError::PasswordNotValid);
        }
        info!("Correct password");

        if verification.hash == MatchStatus::Mismatch {
            info!("Uploading file because remote hash did not match");
            self.transport.send_file(&mut reader, req.size)?;
        } else {
            info!("Remote file already up to date, no upload needed");
        }

        Ok(())
    }
}
