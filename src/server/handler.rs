use crate::{
    protocol::{PasswordMatch, Request},
    server::ServerError,
};
use std::net::TcpStream;
use tracing::{debug, info};

pub struct ClientHandler {
    pub(super) socket: TcpStream,
    pub(super) password: [u8; 32],
}

impl ClientHandler {
    pub fn new(socket: TcpStream, password: [u8; 32]) -> Self {
        Self { socket, password }
    }

    pub fn handle_client(&mut self) -> Result<(), ServerError> {
        let req: Request = self.read_and_decode()?;
        debug!("Received request header: {req:?}");

        if req.password != self.password {
            self.encode_and_write(PasswordMatch::NoMatch)?;
            info!("Passwords did not match!");
            return Err(ServerError::PasswordsDontMatch);
        } else {
            info!("Passwords matched!");
            self.encode_and_write(PasswordMatch::Match)?;
        }

        let hash_match = Self::check_hash(&req.path, req.hash)?;
        self.encode_and_write(hash_match)?;

        Ok(())
    }
}
