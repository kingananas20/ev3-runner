use super::{Transport, TransportError};
use bincode::{config::standard, de::Decode, enc::Encode};
use std::io::{Read, Write};
use tracing::warn;

impl Transport {
    pub fn encode_and_write<T>(&mut self, data: T) -> Result<(), TransportError>
    where
        T: Encode,
    {
        let encoded = bincode::encode_to_vec(data, standard())
            .inspect_err(|e| warn!("Failed to encode the data: {e}"))?;
        let len = encoded.len();
        let size = (len as u32).to_be_bytes();

        self.stream
            .write_all(&size)
            .inspect_err(|e| warn!("Failed to write the data length to the stream: {e}"))?;
        self.stream
            .write_all(&encoded)
            .inspect_err(|e| warn!("Failed to write the data to the stream: {e}"))?;
        Ok(())
    }

    pub fn read_and_decode<T>(&mut self) -> Result<T, TransportError>
    where
        T: Decode<()>,
    {
        let mut len = [0u8; 4];
        self.stream
            .read_exact(&mut len)
            .inspect_err(|e| warn!("Failed to read the data length from the socket: {e}"))?;
        let size = u32::from_be_bytes(len) as usize;

        let mut buf = vec![0u8; size];
        self.stream
            .read_exact(&mut buf)
            .inspect_err(|e| warn!("Failed to read the data from the stream: {e}"))?;

        let (data, _) = bincode::decode_from_slice(&buf, standard())
            .inspect_err(|e| warn!("Failed to decode the data: {e}"))?;
        Ok(data)
    }
}
