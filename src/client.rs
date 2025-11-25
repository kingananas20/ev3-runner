use crate::{
    BUFFER_SIZE,
    cli::{Action, Client},
    hash::Hasher,
    protocol::{self, HashMatch, PasswordMatch, Request},
};
use bincode::{
    config::standard,
    error::{DecodeError, EncodeError},
};
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
    #[error("Passwords not valid")]
    PasswordNotValid,
    #[error("Io error: {0}")]
    Io(#[from] io::Error),
    #[error("Encode error: {0}")]
    Encode(#[from] EncodeError),
    #[error("Decode error: {0}")]
    Decode(#[from] DecodeError),
}

pub fn client(config: Client) -> Result<(), ClientError> {
    let (args, action) = match config.action {
        Action::Upload(args) => (args, protocol::Action::Upload),
        Action::Run(args) => (args, protocol::Action::Run),
    };

    if !args.filepath.is_file() {
        return Err(ClientError::PathNotValid(args.filepath));
    }

    let remote_path = if let Some(remote_path) = args.remote_path {
        remote_path
    } else {
        let Some(remote_path) = args.filepath.file_name() else {
            return Err(ClientError::PathNotValid(args.filepath));
        };
        PathBuf::from(remote_path)
    };

    let file = File::open(&args.filepath)?;
    let file_size = file.metadata()?.len();
    let mut reader = BufReader::new(file);

    let hash = Hasher::hash_file(&mut reader)?;
    reader.rewind()?;

    let password_hash = Hasher::hash_password(&args.password);
    let request = Request {
        action,
        path: remote_path,
        size: file_size,
        hash,
        password: password_hash,
    };
    let request = bincode::encode_to_vec(request, standard())?;
    let req_len_bytes = (request.len() as u32).to_be_bytes();

    let mut stream = TcpStream::connect(&args.host)?;
    info!("Connection established to {}", args.host);

    stream.write_all(&req_len_bytes)?;
    stream.write_all(&request)?;

    let mut pwd_len = [0u8; 4];
    stream.read_exact(&mut pwd_len)?;
    let mut pwd_buf = vec![0u8; u32::from_be_bytes(pwd_len) as usize];
    stream.read_exact(&mut pwd_buf)?;
    let (pwd_match, _): (PasswordMatch, _) = bincode::decode_from_slice(&pwd_buf, standard())?;
    if pwd_match == PasswordMatch::NoMatch {
        return Err(ClientError::PasswordNotValid);
    }
    info!("Correct password");

    let mut res_len = [0u8; 4];
    stream.read_exact(&mut res_len)?;
    let mut buf = vec![0u8; u32::from_be_bytes(res_len) as usize];
    stream.read_exact(&mut buf)?;
    let (hash_match, _): (HashMatch, _) = bincode::decode_from_slice(&buf, standard())?;

    if hash_match == HashMatch::NoMatch {
        let mut remaining = file_size;
        let mut buf = [0u8; BUFFER_SIZE];
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

    let mut stdout = io::stdout();
    let mut buf = [0u8; BUFFER_SIZE];

    loop {
        let n = stream.read(&mut buf)?;
        if n == 0 {
            break;
        }
        stdout.write_all(&buf[..n])?;
        stdout.flush()?;
    }

    info!("Successfully run the file");

    Ok(())
}
