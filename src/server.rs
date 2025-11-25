mod handler;
mod hash;
mod transport;

use crate::BUFFER_SIZE;
use crate::cli::Server;
use crate::hash::Hasher;
use crate::protocol::{Action, HashMatch, PasswordMatch, Request};
use bincode::config::standard;
use bincode::error::{DecodeError, EncodeError};
use handler::ClientHandler;
use std::fs::{File, Permissions};
use std::io::{BufReader, Error};
use std::net::Shutdown;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
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
    #[error("Password hashes don't match")]
    PasswordsDontMatch,
}

pub fn server(config: Server) -> io::Result<()> {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", config.server_port))?;
    info!("Server listening on port {}", config.server_port);

    let password_hash = Hasher::hash_password(&config.password);
    info!("Password hash calculated");

    loop {
        let (socket, addr) = listener.accept()?;
        info!("Accepted connection from {addr}");

        if let Err(e) = handle_client(socket, password_hash) {
            warn!("Error while handling connection: {e}");
        }
    }
}

// todo!!! Create ClientHandler struct and impl the following functions as methods to it
#[tracing::instrument(skip(password_hash))]
fn handle_client(mut socket: TcpStream, password_hash: [u8; 32]) -> Result<(), ServerError> {
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

    if header.password != password_hash {
        let password_res = bincode::encode_to_vec(PasswordMatch::NoMatch, standard())?;
        let password_res_len = u32::to_be_bytes(password_res.len() as u32);
        socket.write_all(&password_res_len)?;
        socket.write_all(&password_res)?;
        socket.shutdown(Shutdown::Both)?;
        return Err(ServerError::PasswordsDontMatch);
    }
    let password_res = bincode::encode_to_vec(PasswordMatch::Match, standard())?;
    let password_res_len = u32::to_be_bytes(password_res.len() as u32);
    socket.write_all(&password_res_len)?;
    socket.write_all(&password_res)?;
    info!("Password match");

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

    if header.action == Action::Upload {
        return Ok(());
    }

    run(&mut socket, &header.path)?;

    Ok(())
}

#[tracing::instrument]
fn check_hash(file_path: &Path, client_hash: u64) -> Result<HashMatch, Error> {
    if !file_path.exists() || !file_path.is_file() {
        return Ok(HashMatch::NoMatch);
    }

    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);

    let hash = Hasher::hash_file(&mut reader)?;

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

    let mut remaining = size;
    let mut buf = [0u8; BUFFER_SIZE];
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

    // only works on linux
    if cfg!(target_os = "linux") {
        file.set_permissions(Permissions::from_mode(0o755))?;
        trace!("Set file permissions on linux");
    }

    info!("File received successfully");

    Ok(())
}

#[tracing::instrument]
fn run(socket: &mut TcpStream, path: &Path) -> Result<(), Error> {
    debug!("Running the file");
    let (mut reader, writer) = std::io::pipe()?;
    let mut child = match Command::new(path)
        .stdout(writer.try_clone()?)
        .stderr(writer)
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            warn!("Failed to spawn child: {e}");
            return Err(e);
        }
    };

    let mut buf = [0u8; BUFFER_SIZE];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        socket.write_all(&buf[..n])?;
    }
    socket.shutdown(std::net::Shutdown::Write)?;

    let status = child.wait()?;
    if status.success() {
        info!("child exited with exitstatus: {status}");
    } else {
        warn!("child exited with exitstatus: {status}");
    }

    info!("Ran the file");

    Ok(())
}
