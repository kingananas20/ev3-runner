mod file_transfer;
mod framed;
mod stream_framer;

use crate::BUFFER_SIZE;
use bincode::error::{DecodeError, EncodeError};
use std::io::{Error, Read, Write};
use std::net::{Shutdown, TcpStream};
use std::ops::{Deref, DerefMut};
use tracing::{trace, warn};

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

    pub fn send_output<R>(&mut self, output: &mut R) -> Result<(), TransportError>
    where
        R: Read,
    {
        let mut buf = [0u8; BUFFER_SIZE];
        loop {
            let n = output.read(&mut buf).map_err(|e| {
                warn!("Failed to read output of the spawned command: {e}");
                e
            })?;
            trace!("n: {n}");
            if n == 0 {
                break;
            }
            self.stream.write_all(&buf[..n]).map_err(|e| {
                warn!("Failed to write output to the stream: {e}");
                e
            })?;
        }

        Ok(())
    }

    pub fn receive_output<W>(&mut self, output: &mut W) -> Result<(), TransportError>
    where
        W: Write,
    {
        let mut buf = [0u8; BUFFER_SIZE];
        loop {
            let n = self.stream.read(&mut buf).map_err(|e| {
                warn!("Failed to read from the stream: {e}");
                e
            })?;
            if n == 0 {
                break;
            }
            output.write_all(&buf[..n]).map_err(|e| {
                warn!("Failed to write to the output: {e}");
                e
            })?;
            output.flush().map_err(|e| {
                warn!("Failed to flush the output: {e}");
                e
            })?;
        }

        Ok(())
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
