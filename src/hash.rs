use crate::BUFFER_SIZE;
use std::{
    fs::File,
    hash::Hasher as _,
    io::{BufReader, Error, Read},
};
use twox_hash::XxHash64;

pub fn calculate_hash(file: &mut BufReader<File>) -> Result<u64, Error> {
    let seed = 4167;
    let mut hasher = XxHash64::with_seed(seed);

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
