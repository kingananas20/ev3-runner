use crate::BUFFER_SIZE;
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    hash::Hasher as _,
    io::{BufReader, Error, Read},
};
use twox_hash::XxHash64;

pub struct Hasher;

impl Hasher {
    const SEED: u64 = 4167;

    pub fn hash_file(file: &mut BufReader<File>) -> Result<u64, Error> {
        let mut hasher = XxHash64::with_seed(Self::SEED);

        let mut buf = [0u8; BUFFER_SIZE];

        loop {
            let n = file.read(&mut buf)?;
            if n == 0 {
                break;
            }
            hasher.write(&buf[..n]);
        }

        Ok(hasher.finish())
    }

    pub fn hash_password(password: &str) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(password);
        hasher.finalize().into()
    }
}
