mod validate_path;

use super::{ClientHandler, handler::HandlerError};
use crate::protocol::{MatchStatus, PathStatus, Request, Validation};
use std::path::PathBuf;
use tracing::{info, warn};
use validate_path::validate_path;

impl ClientHandler {
    pub(super) fn validation(
        &mut self,
        req: &Request,
    ) -> Result<(Validation, PathBuf), HandlerError> {
        let mut validation = Validation::default();

        if req.password != self.password {
            self.transport.encode_and_write(validation)?;
            info!("Passwords did not match!");
            return Err(HandlerError::PasswordsDontMatch);
        } else {
            validation.password = MatchStatus::Match;
            info!("Passwords matched!");
        }

        let (safe_path, path_status) = match validate_path(&req.path) {
            Ok(sp) => {
                info!("Path is valid");
                (sp, PathStatus::Valid)
            }
            Err(e) => {
                warn!("Path is not valid: {e}");
                validation.path = e;
                self.transport.encode_and_write(validation)?;
                return Err(e.into());
            }
        };
        validation.path = path_status;

        validation.hash = Self::check_hash(&req.path, req.hash)?;
        self.transport.encode_and_write(validation)?;

        Ok((validation, safe_path))
    }
}
