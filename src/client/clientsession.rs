use crate::{
    cli::ClientArgs,
    hash::Hasher,
    protocol::{Action, MatchStatus, Request, Verification},
    transport::{Transport, TransportError},
};
use bincode::error::{DecodeError, EncodeError};
use std::{
    fs::File,
    io::{self, BufReader, Seek},
    net::Shutdown,
    path::PathBuf,
};
use tracing::{error, info};

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("Path does not point to a file or the file doesn't exist: {0}")]
    PathNotValid(PathBuf),
    #[error("Passwords not valid")]
    PasswordNotValid,
    #[error("Version mismatch: {0}")]
    VersionMismatch(String),
    #[error("Error in transport layer: {0}")]
    Transport(#[from] TransportError),
    #[error("Io error: {0}")]
    Io(#[from] io::Error),
    #[error("Encode error: {0}")]
    Encode(#[from] EncodeError),
    #[error("Decode error: {0}")]
    Decode(#[from] DecodeError),
}

pub struct ClientSession {
    pub(super) transport: Transport,
    pub(super) args: ClientArgs,
    pub(super) action: Action,
}

impl ClientSession {
    pub fn connect(args: ClientArgs, action: Action) -> Result<Self, ClientError> {
        let transport = Transport::connect(&args.host)?;
        Ok(Self {
            transport,
            args,
            action,
        })
    }

    pub fn dispatch(&mut self) -> Result<(), ClientError> {
        self.check_version()?;

        let (req, mut reader) = self.setup()?;
        self.transport.encode_and_write(&req)?;

        let verification = self.transport.read_and_decode::<Verification>()?;

        if verification.password == MatchStatus::Mismatch {
            return Err(ClientError::PasswordNotValid);
        }
        info!("Password correct");

        if verification.hash == MatchStatus::Mismatch {
            info!("Uploading file because remote hash did not match");
            self.transport.send_file(&mut reader, req.size)?;
        } else {
            info!("Remote file already up to date, no upload needed");
        }

        if self.action == Action::Upload {
            info!("Done with this session");
            return Ok(());
        }

        let mut stdout = io::stdout();
        self.transport.receive_output(&mut stdout)?;

        self.transport.stream.shutdown(Shutdown::Both)?;
        info!("Done with this session");

        Ok(())
    }

    fn setup(&mut self) -> Result<(Request, BufReader<File>), ClientError> {
        if !self.args.filepath.is_file() {
            return Err(ClientError::PathNotValid(self.args.filepath.clone()));
        }

        let remote_path = self.args.remote_path.clone().map(Ok).unwrap_or_else(
            || -> Result<PathBuf, ClientError> {
                self.args
                    .filepath
                    .file_name()
                    .map(PathBuf::from)
                    .ok_or_else(|| ClientError::PathNotValid(self.args.filepath.clone()))
            },
        )?;

        let file = File::open(&self.args.filepath)?;
        let file_size = file.metadata()?.len();
        let mut reader = BufReader::new(file);

        let hash = Hasher::hash_file(&mut reader)?;
        reader.rewind()?;

        let password_hash = Hasher::hash_password(&self.args.password);
        let request = Request {
            action: self.action,
            path: remote_path,
            size: file_size,
            hash,
            password: password_hash,
        };

        Ok((request, reader))
    }
}
