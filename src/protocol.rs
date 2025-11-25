use bincode::{Decode, Encode};
use std::{fmt::Debug, path::PathBuf};

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
    Run,
}

#[derive(Debug, Decode, Encode, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum PasswordMatch {
    Match,
    NoMatch,
}

#[derive(Debug, Decode, Encode, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum HashMatch {
    Match,
    NoMatch,
}
