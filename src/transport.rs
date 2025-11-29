use crate::BUFFER_SIZE;
use bincode::error::{DecodeError, EncodeError};
use bincode::{Encode, config::standard, de::Decode};
use socket2::Socket;
use std::net::{Shutdown, TcpStream};
use std::ops::{Deref, DerefMut};
use std::time::Instant;
use std::{
    cmp,
    io::{Error, Read, Write},
};
use tracing::{debug, trace, warn};

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
        Self::set_buffers(&stream).unwrap();
        Self { stream }
    }

    pub fn connect(addr: &str) -> Result<Self, TransportError> {
        let stream = TcpStream::connect(addr)?;
        stream.set_nodelay(true)?;
        Self::set_buffers(&stream)?;
        Ok(Self { stream })
    }

    const SOCK_BUFFER_SIZE: usize = 256 * 1024;

    fn set_buffers(stream: &TcpStream) -> Result<(), TransportError> {
        let socket = Socket::from(stream.try_clone()?);
        socket.set_send_buffer_size(Self::SOCK_BUFFER_SIZE)?;
        socket.set_recv_buffer_size(Self::SOCK_BUFFER_SIZE)?;
        Ok(())
    }

    pub fn encode_and_write<T>(&mut self, data: T) -> Result<(), TransportError>
    where
        T: Encode,
    {
        let encoded = bincode::encode_to_vec(data, standard()).map_err(|e| {
            warn!("Failed to encode the data: {e}");
            e
        })?;
        let len = encoded.len();
        let size = (len as u32).to_be_bytes();

        self.stream.write_all(&size).map_err(|e| {
            warn!("Failed to write the data length to the stream: {e}");
            e
        })?;
        self.stream.write_all(&encoded).map_err(|e| {
            warn!("Failed to write the data to the stream: {e}");
            e
        })?;
        Ok(())
    }

    pub fn read_and_decode<T>(&mut self) -> Result<T, TransportError>
    where
        T: Decode<()>,
    {
        let mut len = [0u8; 4];
        self.stream.read_exact(&mut len).map_err(|e| {
            warn!("Failed to read the data length from the socket: {e}");
            e
        })?;
        let size = u32::from_be_bytes(len) as usize;

        let mut buf = vec![0u8; size];
        self.stream.read_exact(&mut buf).map_err(|e| {
            warn!("Failed to read the data from the stream: {e}");
            e
        })?;

        let (data, _) = bincode::decode_from_slice(&buf, standard()).map_err(|e| {
            warn!("Failed to decode the data: {e}");
            e
        })?;
        Ok(data)
    }

    pub fn send_file<R>(&mut self, file: &mut R, size: u64) -> Result<(), TransportError>
    where
        R: Read,
    {
        let mut buf = [0u8; BUFFER_SIZE];
        let mut remaining = size as usize;

        while remaining > 0 {
            let to_read = cmp::min(remaining, buf.len());

            file.read_exact(&mut buf[..to_read]).map_err(|e| {
                warn!("Failed to read the file from disk: {e}");
                e
            })?;

            self.stream.write_all(&buf[..to_read]).map_err(|e| {
                warn!("Failed to write File to stream: {e}");
                e
            })?;

            remaining -= to_read;
            trace!("remaining: {remaining} / read: {to_read}");
        }

        Ok(())
    }

    pub fn receive_file<W>(&mut self, file: &mut W, size: u64) -> Result<(), TransportError>
    where
        W: Write,
    {
        let instant = Instant::now();
        let mut buf = [0u8; BUFFER_SIZE];
        let mut remaining = size as usize;

        while remaining > 0 {
            let i1 = Instant::now();
            let n = self.stream.read(&mut buf).map_err(|e| {
                warn!("Failed to read the file from the stream: {e}");
                e
            })?;
            if n == 0 {
                break;
            }
            let i2 = Instant::now();
            file.write_all(&buf[..n]).map_err(|e| {
                warn!("Failed to write file content to disk: {e}");
                e
            })?;
            let i3 = Instant::now();
            remaining -= n;
            trace!("remaining: {remaining} / read: {n}");
            trace!("read took: {:?}, write took {:?}", i2 - i1, i3 - i2);
        }

        debug!("Took {:?}", instant.elapsed());

        Ok(())
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
