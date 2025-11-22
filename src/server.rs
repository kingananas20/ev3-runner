use crate::cli::Server;
use crate::protocol::{Action, Request};
use std::{
    fs::OpenOptions,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    path::Path,
};
use tracing::{debug, error, info};

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
fn handle_client(mut socket: TcpStream) -> io::Result<()> {
    let mut len_buf = [0u8; 4];
    if let Err(e) = socket.read_exact(&mut len_buf) {
        error!("Failed to read header length");
        return Err(e);
    }
    let header_len = u32::from_be_bytes(len_buf) as usize;

    let mut header_buf = vec![0u8; header_len];
    if let Err(e) = socket.read_exact(&mut header_buf) {
        error!("Failed to read header");
        return Err(e);
    }

    let header: Request = match bincode::decode_from_slice(&header_buf, bincode::config::standard())
    {
        Ok((h, _)) => h,
        Err(e) => {
            error!("Failed to deserialize header: {e}");
            return Ok(());
        }
    };

    match header.action {
        Action::Upload => upload(&mut socket, &header.path, header.size)?,
        Action::Run => upload(&mut socket, &header.path, header.size)?,
    }

    Ok(())
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
            error!("Failed to open file: {e}");
            return Err(e);
        }
    };

    let mut remaining = size;
    let mut buf = [0u8; 8 * 1024];
    while remaining > 0 {
        let to_read = std::cmp::min(remaining, buf.len() as u64) as usize;
        if let Err(e) = socket.read_exact(&mut buf[..to_read]) {
            error!("Failed to read to buffer: {e}");
            return Err(e);
        };

        if let Err(e) = file.write_all(&buf[..to_read]) {
            error!("Failed to write to file");
            return Err(e);
        }

        remaining -= to_read as u64;
    }

    info!("File received successfully");

    Ok(())
}
