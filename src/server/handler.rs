use crate::{
    protocol::{Action, MatchStatus, PathStatus, Request},
    transport::{Transport, TransportError},
};
use bincode::error::{DecodeError, EncodeError};
use std::{io::Error, net::TcpStream};
use tracing::{debug, info};

pub struct ClientHandler {
    pub(super) transport: Transport,
    pub(super) password: [u8; 32],
}

impl ClientHandler {
    pub fn new(socket: TcpStream, password: [u8; 32]) -> Self {
        let transport = Transport::new(socket);
        Self {
            transport,
            password,
        }
    }

    pub fn handle_client(&mut self) -> Result<(), HandlerError> {
        self.check_version()?;

        let req: Request = self.transport.read_and_decode()?;
        debug!("Received request header: {req:?}");

        let (validation, safe_path) = self.validation(&req)?;

        if validation.hash == MatchStatus::Mismatch {
            self.download(&safe_path, req.use_compression)?;
            info!("File received successfully");
        }

        #[cfg(unix)]
        self.set_permissions(&safe_path)?;

        if req.action == Action::Upload {
            info!("Done with this client");
            return Ok(());
        }

        if let Action::Run(brickrun) = req.action {
            self.run(&safe_path, brickrun)?;
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HandlerError {
    #[error("Error in the transport layer: {0}")]
    Transport(#[from] TransportError),
    #[error("Encode error: {0}")]
    Encode(#[from] EncodeError),
    #[error("Decode error: {0}")]
    Decode(#[from] DecodeError),
    #[error("Io error: {0}")]
    Io(#[from] Error),
    #[error("Password hashes don't match")]
    PasswordsDontMatch,
    #[error("Version mismatch: {0}")]
    VersionMismatch(String),
    #[error("Path validation error: {0}")]
    PathValidation(#[from] PathStatus),
}
