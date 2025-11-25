use super::{ClientHandler, ServerError};
use bincode::{Encode, config::standard, de::Decode};
use std::io::{Read, Write};
use tracing::warn;

impl ClientHandler {
    pub(super) fn read_and_decode<T>(&mut self) -> Result<T, ServerError>
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

    pub(super) fn encode_and_write<T>(&mut self, data: T) -> Result<(), ServerError>
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
}
