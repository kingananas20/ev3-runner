use bincode::{Decode, Encode};
use std::{fmt::Debug, path::PathBuf};

#[derive(Debug, Decode, Encode, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct VersionHeader(pub String);

#[derive(Debug, Decode, Encode, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct VersionResponse(pub VersionStatus);

#[derive(Debug, Decode, Encode, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum VersionStatus {
    Match,
    Mismatch(String),
}

#[derive(Decode, Encode, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Request {
    pub action: Action,
    pub path: PathBuf,
    pub size: u64,
    pub hash: u64,
    pub password: [u8; 32],
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
    Run(bool),
}

#[derive(Debug, Decode, Encode, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Validation {
    pub password: MatchStatus,
    pub hash: MatchStatus,
    pub path: PathStatus,
}

impl Default for Validation {
    fn default() -> Self {
        Self {
            password: MatchStatus::Mismatch,
            hash: MatchStatus::Mismatch,
            path: PathStatus::Valid,
        }
    }
}

#[derive(Debug, Decode, Encode, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum MatchStatus {
    Match,
    Mismatch,
}

#[derive(Debug, Decode, Encode, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, thiserror::Error)]
pub enum PathStatus {
    #[error("Path is valid. This isn't an error")]
    Valid,
    #[error("Path contains invalid components (e.g., '..')")]
    InvalidComponents,
    #[error("Path is absolute, only relative paths allowed")]
    AbsolutePath,
    #[error("Path would escape working directory")]
    EscapesWorkingDir,
    #[error("Failed to canonicalize path")]
    CanonicalizationFailed,
}
