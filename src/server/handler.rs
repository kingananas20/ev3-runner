use crate::{
    VERSION,
    protocol::{Action, MatchStatus, Request, Verification, VersionStatus},
    transport::{Transport, TransportError},
};
use bincode::error::{DecodeError, EncodeError};
use std::{
    io::Error,
    net::{Shutdown, TcpStream},
};
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
        let req: Request = self.transport.read_and_decode()?;
        debug!("Received request header: {req:?}");

        let mut verification = Verification::default();

        if req.version != VERSION {
            verification.version = VersionStatus::Mismatch(VERSION.to_owned());
            self.transport.encode_and_write(verification)?;
            self.transport.shutdown(Shutdown::Both)?;
            return Err(HandlerError::VersionMismatch(VERSION.to_owned()));
        }

        if req.password != self.password {
            self.transport.encode_and_write(verification)?;
            self.transport.shutdown(Shutdown::Both)?;
            info!("Passwords did not match!");
            return Err(HandlerError::PasswordsDontMatch);
        } else {
            verification.password = MatchStatus::Match;
            info!("Passwords matched!");
        }

        verification.hash = Self::check_hash(&req.path, req.hash)?;
        if verification.hash == MatchStatus::Mismatch {
            self.upload(&req.path, req.size)?;
            info!("File received successfully");
        }

        if req.action == Action::Upload {
            info!("Done with this client");
            self.transport.shutdown(Shutdown::Both)?;
            return Ok(());
        }

        self.run(&req.path)?;

        self.transport.shutdown(Shutdown::Both)?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub(super) enum HandlerError {
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
}
