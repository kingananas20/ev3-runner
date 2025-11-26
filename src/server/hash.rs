use super::ClientHandler;
use crate::{hash::Hasher, protocol::MatchStatus};
use std::{
    fs::File,
    io::{self, BufReader},
    path::Path,
};
use tracing::{debug, info, warn};

impl ClientHandler {
    pub(super) fn check_hash(path: &Path, remote_hash: u64) -> Result<MatchStatus, io::Error> {
        if !path.exists() || path.is_dir() {
            return Ok(MatchStatus::Mismatch);
        }

        let file = File::open(path).map_err(|e| {
            warn!("Failed to open the file: {e}");
            e
        })?;
        let mut reader = BufReader::new(file);

        let hash = Hasher::hash_file(&mut reader).map_err(|e| {
            warn!("Failed to calculate hash of the file: {e}");
            e
        })?;

        debug!("hash: {hash} / remote_hash: {remote_hash}");
        if hash != remote_hash {
            info!("Hashes don't match");
            return Ok(MatchStatus::Mismatch);
        }

        info!("Hashes match");
        Ok(MatchStatus::Match)
    }
}
