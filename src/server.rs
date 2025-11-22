use crate::cli::Server;
use crate::hash::calculate_hash;
use crate::protocol::{HashMatch, Request};
use bincode::config::standard;
use bincode::error::{DecodeError, EncodeError};
use std::fs::{File, Permissions};
use std::io::{BufReader, Error};
use std::os::unix::fs::PermissionsExt;
use std::{
    fs::OpenOptions,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    path::Path,
};
use tracing::{debug, error, info, trace, warn};

#[derive(Debug, thiserror::Error)]
enum ServerError {
    #[error("Encode error: {0}")]
    Encode(#[from] EncodeError),
    #[error("Decode error: {0}")]
    Decode(#[from] DecodeError),
    #[error("Io error: {0}")]
    Io(#[from] Error),
}

pub fn server(config: Server) -> io::Result<()> {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", config.server_port))?;
    info!("Server listening on port {}", config.server_port);

    loop {
        let (socket, addr) = listener.accept()?;
        info!("Accepted connection from {addr}");

        handle_client(socket).ok();
    }
}

#[tracing::instrument]
fn handle_client(mut socket: TcpStream) -> Result<(), ServerError> {
    let mut len_buf = [0u8; 4];
    if let Err(e) = socket.read_exact(&mut len_buf) {
        warn!("Failed to read header length");
        return Err(e.into());
    }
    let header_len = u32::from_be_bytes(len_buf) as usize;
    trace!("header_len: {header_len}");

    let mut header_buf = vec![0u8; header_len];
    if let Err(e) = socket.read_exact(&mut header_buf) {
        warn!("Failed to read header");
        return Err(e.into());
    }

    let header: Request = match bincode::decode_from_slice(&header_buf, bincode::config::standard())
    {
        Ok((h, _)) => h,
        Err(e) => {
            warn!("Failed to deserialize header: {e}");
            return Err(e.into());
        }
    };
    trace!("header: {header:?}");

    let hash_match = check_hash(&header.path, header.hash)?;
    info!("Does the hash match? {:?}", hash_match);
    let res = bincode::encode_to_vec(hash_match, standard())?;
    let res_len = (res.len() as u32).to_be_bytes();
    trace!("res_len: {res_len:?}");
    socket.write_all(&res_len)?;
    socket.write_all(&res)?;

    if hash_match == HashMatch::NoMatch {
        upload(&mut socket, &header.path, header.size)?;
    }

    Ok(())
}

#[tracing::instrument]
fn check_hash(file_path: &Path, client_hash: u64) -> Result<HashMatch, Error> {
    if !file_path.exists() || !file_path.is_file() {
        return Ok(HashMatch::NoMatch);
    }

    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);

    let hash = calculate_hash(&mut reader)?;

    debug!("hash: {hash} / client_hash: {client_hash}");
    if hash == client_hash {
        return Ok(HashMatch::Match);
    }

    Ok(HashMatch::NoMatch)
}

#[tracing::instrument]
fn upload(socket: &mut TcpStream, path: &Path, size: u64) -> io::Result<()> {
    debug!("Receiving file");

    let mut file = match OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
    {
        Ok(f) => f,
        Err(e) => {
            warn!("Failed to open file: {e}");
            return Err(e);
        }
    };

    // only works on linux
    if cfg!(target_os = "linux") {
        file.set_permissions(Permissions::from_mode(0o755))?;
        trace!("Set file permissions on linux");
    }

    let mut remaining = size;
    let mut buf = [0u8; 8 * 1024];
    while remaining > 0 {
        let to_read = std::cmp::min(remaining, buf.len() as u64) as usize;
        if let Err(e) = socket.read_exact(&mut buf[..to_read]) {
            warn!("Failed to read to buffer: {e}");
            return Err(e);
        };

        if let Err(e) = file.write_all(&buf[..to_read]) {
            warn!("Failed to write to file");
            return Err(e);
        }

        remaining -= to_read as u64;
        trace!("remaining: {remaining} / to_read: {to_read}");
    }

    info!("File received successfully");

    Ok(())
}
