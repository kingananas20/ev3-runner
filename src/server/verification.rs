use crate::protocol::{MatchStatus, Request, Verification};
use crate::server::handler::{ClientHandler, HandlerError};
use tracing::info;

impl ClientHandler {
    pub(super) fn verification(&mut self, req: &Request) -> Result<(), HandlerError> {
        let mut verification = Verification::default();

        if req.password != self.password {
            self.transport.encode_and_write(verification)?;
            info!("Passwords did not match!");
            return Err(HandlerError::PasswordsDontMatch);
        } else {
            verification.password = MatchStatus::Match;
            info!("Passwords matched!");
        }

        verification.hash = Self::check_hash(&req.path, req.hash)?;
        self.transport.encode_and_write(&verification)?;

        if verification.hash == MatchStatus::Mismatch {
            self.upload(&req.path, req.size)?;
            info!("File received successfully");
        }

        #[cfg(unix)]
        self.set_permissions(&req.path)?;

        Ok(())
    }
}
