mod handler;
mod hash;
mod run;
mod upload;
mod version;

use crate::cli::Server;
use crate::hash::Hasher;
use handler::ClientHandler;
use std::{
    io::{self},
    net::TcpListener,
};
use tracing::{info, warn};

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
