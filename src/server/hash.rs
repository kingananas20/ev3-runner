use super::ClientHandler;
use crate::{hash::Hasher, protocol::MatchStatus};
use std::{
    fs::File,
    io::{self, BufReader},
    path::Path,
};
use tracing::{debug, trace, warn};

impl ClientHandler {
    pub(super) fn check_hash(path: &Path, remote_hash: u64) -> Result<MatchStatus, io::Error> {
        if !path.exists() || path.is_dir() {
            return Ok(MatchStatus::Mismatch);
        }

        let file = File::open(path).inspect_err(|e| warn!("Failed to open the file: {e}"))?;
        let mut reader = BufReader::new(file);

        let hash = Hasher::hash_file(&mut reader)
            .inspect_err(|e| warn!("Failed to calculate hash of the file: {e}"))?;

        trace!("hash: {hash} / remote_hash: {remote_hash}");
        if hash != remote_hash {
            debug!("Hashes don't match");
            return Ok(MatchStatus::Mismatch);
        }

        debug!("Hashes match");
        Ok(MatchStatus::Match)
    }
}
