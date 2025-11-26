use bincode::{Decode, Encode};
use std::{fmt::Debug, path::PathBuf};

#[derive(Decode, Encode, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Request {
    pub action: Action,
    pub path: PathBuf,
    pub size: u64,
    pub hash: u64,
    pub password: [u8; 32],
    pub version: String,
}

impl Debug for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Request")
            .field("action", &self.action)
            .field("path", &self.path)
            .field("size", &self.size)
            .field("hash", &self.hash)
            .field("password", &"REDACTED")
            .finish()
    }
}

#[derive(Debug, Decode, Encode, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Action {
    Upload,
    Run,
}

#[derive(Debug, Decode, Encode, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Verification {
    pub password: MatchStatus,
    pub hash: MatchStatus,
    pub version: VersionStatus,
}

impl Default for Verification {
    fn default() -> Self {
        Self {
            password: MatchStatus::Mismatch,
            hash: MatchStatus::Mismatch,
            version: VersionStatus::Mismatch("default".to_owned()),
        }
    }
}

#[derive(Debug, Decode, Encode, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum MatchStatus {
    Match,
    Mismatch,
}

#[derive(Debug, Decode, Encode, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum VersionStatus {
    Match,
    Mismatch(String),
}
