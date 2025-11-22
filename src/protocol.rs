use bincode::{Decode, Encode};
use std::path::PathBuf;

#[derive(Debug, Decode, Encode, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Request {
    pub action: Action,
    pub path: PathBuf,
    pub size: u64,
    pub hash: u64,
}

#[derive(Debug, Decode, Encode, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Action {
    Upload,
    Run,
}
