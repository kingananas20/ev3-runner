use crate::protocol::{MatchStatus, PathStatus, Request, Validation};
use crate::server::handler::{ClientHandler, HandlerError};
use std::env;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

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

        let (safe_path, path_status) = match Self::validate_path(&req.path) {
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

    pub(super) fn validate_path(path: &Path) -> Result<PathBuf, PathStatus> {
        if path.is_absolute() {
            return Err(PathStatus::AbsolutePath);
        }

        if path
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            return Err(PathStatus::InvalidComponents);
        }

        let working_dir = env::current_dir().map_err(|_| PathStatus::CanonicalizationFailed)?;

        let target_path = working_dir.join(path);

        let canonical_target = if target_path.exists() {
            target_path
                .canonicalize()
                .map_err(|_| PathStatus::CanonicalizationFailed)?
        } else {
            let parent = target_path.parent().unwrap_or(&working_dir);

            if parent.exists() {
                let canonical_parent = parent
                    .canonicalize()
                    .map_err(|_| PathStatus::CanonicalizationFailed)?;
                let file_name = match target_path.file_name() {
                    Some(f) => f,
                    None => return Err(PathStatus::InvalidComponents),
                };
                canonical_parent.join(file_name)
            } else {
                working_dir.join(path)
            }
        };

        if !canonical_target.starts_with(&working_dir) {
            return Err(PathStatus::EscapesWorkingDir);
        }

        let safe_path = canonical_target
            .strip_prefix(&working_dir)
            .expect("Panicking is impossible")
            .to_path_buf();
        info!("Safe path: {}", safe_path.display());

        Ok(safe_path)
    }
}
