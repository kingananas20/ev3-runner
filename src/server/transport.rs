use super::ClientHandler;
use crate::{BUFFER_SIZE, server::handler::HandlerError};
use bincode::{Encode, config::standard, de::Decode};
use std::{
    cmp,
    io::{Error, Read, Write},
};
use tracing::{trace, warn};

impl ClientHandler {
    pub(super) fn read_and_decode<T>(&mut self) -> Result<T, HandlerError>
    where
        T: Decode<()>,
    {
        let mut len = [0u8; 4];
        self.socket.read_exact(&mut len).map_err(|e| {
            warn!("Failed to read the data length from the socket: {e}");
            e
        })?;
        let size = u32::from_be_bytes(len) as usize;

        let mut buf = vec![0u8; size];
        self.socket.read_exact(&mut buf).map_err(|e| {
            warn!("Failed to read the data from the socket: {e}");
            e
        })?;

        let (data, _) = bincode::decode_from_slice(&buf, standard()).map_err(|e| {
            warn!("Failed to decode the data: {e}");
            e
        })?;
        Ok(data)
    }

    pub(super) fn encode_and_write<T>(&mut self, data: T) -> Result<(), HandlerError>
    where
        T: Encode,
    {
        let encoded = bincode::encode_to_vec(data, standard()).map_err(|e| {
            warn!("Failed to encode the data: {e}");
            e
        })?;
        let len = encoded.len();
        let size = (len as u32).to_be_bytes();

        self.socket.write_all(&size).map_err(|e| {
            warn!("Failed to write the data length to the socket: {e}");
            e
        })?;
        self.socket.write_all(&encoded).map_err(|e| {
            warn!("Failed to write the data to the socket: {e}");
            e
        })?;
        Ok(())
    }

    pub(super) fn receive_file<W>(&mut self, file: &mut W, size: u64) -> Result<(), Error>
    where
        W: Write,
    {
        let mut buf = [0u8; BUFFER_SIZE];
        let mut remaining = size as usize;

        while remaining > 0 {
            let to_read = cmp::min(remaining, BUFFER_SIZE);

            self.socket.read_exact(&mut buf[..to_read]).map_err(|e| {
                warn!("Failed to read the file from the socket: {e}");
                e
            })?;

            file.write_all(&buf[..to_read]).map_err(|e| {
                warn!("Failed to write file content to disk: {e}");
                e
            })?;

            remaining -= to_read;
            trace!("remaining: {remaining} / read: {to_read}");
        }

        Ok(())
    }

    pub(super) fn send_output<R>(&mut self, output: &mut R) -> Result<(), Error>
    where
        R: Read,
    {
        let mut buf = [0u8; BUFFER_SIZE];
        loop {
            let n = output.read(&mut buf).map_err(|e| {
                warn!("Failed to read output of the spawned command: {e}");
                e
            })?;
            if n == 0 {
                break;
            }
            self.socket.write_all(&buf[..n]).map_err(|e| {
                warn!("Failed to write output to the socket: {e}");
                e
            })?;
        }

        Ok(())
    }
}
