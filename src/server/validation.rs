use crate::protocol::{MatchStatus, PathStatus, Request, Validation};
use crate::server::handler::{ClientHandler, HandlerError};
use std::env;
use std::io::Error;
use std::path::Path;
use tracing::{info, warn};

impl ClientHandler {
    pub(super) fn validation(&mut self, req: &Request) -> Result<Validation, HandlerError> {
        let mut validation = Validation::default();

        if req.password != self.password {
            self.transport.encode_and_write(validation)?;
            info!("Passwords did not match!");
            return Err(HandlerError::PasswordsDontMatch);
        } else {
            validation.password = MatchStatus::Match;
            info!("Passwords matched!");
        }

        validation.path = Self::validate_path(&req.path)?;
        if validation.path != PathStatus::Valid {
            self.transport.encode_and_write(validation)?;
            warn!("Path is not valid: {}", validation.path);
            return Err(validation.path.into());
        }

        validation.hash = Self::check_hash(&req.path, req.hash)?;
        self.transport.encode_and_write(validation)?;

        Ok(validation)
    }

    pub(super) fn validate_path(path: &Path) -> Result<PathStatus, Error> {
        if path.is_absolute() {
            return Ok(PathStatus::AbsolutePath);
        }

        if path
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            return Ok(PathStatus::InvalidComponents);
        }

        let working_dir = env::current_dir()?;

        let target_path = working_dir.join(path);

        let canonical_target = if target_path.exists() {
            target_path.canonicalize()?
        } else {
            let parent = target_path.parent().unwrap_or(&working_dir);

            if parent.exists() {
                let canonical_parent = parent.canonicalize()?;
                let file_name = match target_path.file_name() {
                    Some(f) => f,
                    None => return Ok(PathStatus::InvalidComponents),
                };
                canonical_parent.join(file_name)
            } else {
                working_dir.join(path)
            }
        };

        if !canonical_target.starts_with(&working_dir) {
            return Ok(PathStatus::EscapesWorkingDir);
        }

        Ok(PathStatus::Valid)
    }
}
