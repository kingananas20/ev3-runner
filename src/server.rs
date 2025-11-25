mod handler;
mod hash;
mod transport;
mod upload;

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

        let mut client_handler = ClientHandler::new(socket, password_hash);
        if let Err(e) = client_handler.handle_client() {
            warn!("Error while handling connection: {e}");
        }
    }
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
