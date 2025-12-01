mod file_transfer;
mod framed;
mod process_stream;
mod stream_framer;

use bincode::error::{DecodeError, EncodeError};
use std::io::{Error, Write};
use std::net::{Shutdown, TcpStream};
use std::ops::{Deref, DerefMut};

#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("Io error: {0}")]
    Io(#[from] Error),
    #[error("Decode error: {0}")]
    Decode(#[from] DecodeError),
    #[error("Encode error: {0}")]
    Encode(#[from] EncodeError),
}

pub struct Transport {
    pub stream: TcpStream,
}

impl Transport {
    pub fn new(stream: TcpStream) -> Self {
        stream.set_nodelay(true).unwrap();
        Self { stream }
    }

    pub fn connect(addr: &str) -> Result<Self, TransportError> {
        let stream = TcpStream::connect(addr)?;
        stream.set_nodelay(true)?;
        Ok(Self { stream })
    }
}

impl Deref for Transport {
    type Target = TcpStream;

    fn deref(&self) -> &Self::Target {
        &self.stream
    }
}

impl DerefMut for Transport {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stream
    }
}

impl Drop for Transport {
    fn drop(&mut self) {
        self.stream.flush().ok();
        self.stream.shutdown(Shutdown::Both).ok();
    }
}
