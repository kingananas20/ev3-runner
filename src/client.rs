use crate::{
    cli::{Action, Client},
    hash::calculate_hash,
    protocol::{self, Request},
};
use bincode::{config::standard, error::EncodeError};
use std::{
    fs::File,
    io::{self, BufReader, Read, Seek, Write},
    net::TcpStream,
    path::PathBuf,
};
use tracing::info;

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("Path does not point to a file or the file doesn't exist: {0}")]
    PathNotValid(PathBuf),
    #[error("Io error: {0}")]
    Io(#[from] io::Error),
    #[error("Error while encoding: {0}")]
    Encode(#[from] EncodeError),
}

pub fn client(config: Client) -> Result<(), ClientError> {
    let (args, action) = match config.action {
        Action::Upload(args) => (args, protocol::Action::Upload),
        Action::Run(args) => (args, protocol::Action::Run),
    };

    if !args.filepath.is_file() {
        return Err(ClientError::PathNotValid(args.filepath));
    }

    let Some(remote_path) = args.filepath.file_name() else {
        return Err(ClientError::PathNotValid(args.filepath));
    };
    let remote_path = PathBuf::from(remote_path);

    let file = File::open(&args.filepath)?;
    let file_size = file.metadata()?.len();
    let mut reader = BufReader::new(file);

    let hash = calculate_hash(&mut reader)?;
    reader.rewind()?;

    let request = Request {
        action,
        path: remote_path,
        size: file_size,
        hash,
    };
    let request = bincode::encode_to_vec(request, standard())?;
    let req_len_bytes = (request.len() as u32).to_be_bytes();

    let mut stream = TcpStream::connect(&args.host)?;
    info!("Connection established to {}", args.host);

    stream.write_all(&req_len_bytes)?;
    stream.write_all(&request)?;

    // Wait for send_file request

    {
        let mut remaining = file_size;
        let mut buf = [0u8; 8 * 1024];
        while remaining > 0 {
            let to_read = std::cmp::min(remaining, buf.len() as u64) as usize;
            reader.read_exact(&mut buf[..to_read])?;
            stream.write_all(&buf[..to_read])?;
            remaining -= to_read as u64;
        }
    }

    if action == protocol::Action::Upload {
        info!("Successfully uploaded file");
        return Ok(());
    }

    Ok(())
}
