use super::{Transport, TransportError};
use crate::BUFFER_SIZE;
use std::io::{Read, Write};
use tracing::{debug, warn};

impl Transport {
    pub fn forward_output<R>(&mut self, output: &mut R) -> Result<(), TransportError>
    where
        R: Read,
    {
        let mut bytes = 0usize;
        let mut buf = [0u8; BUFFER_SIZE];

        loop {
            let n = output.read(&mut buf).map_err(|e| {
                warn!("Failed to read output of the spawned command: {e}");
                e
            })?;

            if n == 0 {
                break;
            }
            bytes += n;

            self.stream.write_all(&buf[..n]).map_err(|e| {
                warn!("Failed to write output to the stream: {e}");
                e
            })?;
        }

        debug!("Streamed output to stream: {bytes} bytes");

        Ok(())
    }

    pub fn receive_output<W>(&mut self, output: &mut W) -> Result<(), TransportError>
    where
        W: Write,
    {
        let mut bytes = 0usize;
        let mut buf = [0u8; BUFFER_SIZE];

        loop {
            let n = self.stream.read(&mut buf).map_err(|e| {
                warn!("Failed to read from the stream: {e}");
                e
            })?;

            if n == 0 {
                break;
            }
            bytes += n;

            output.write_all(&buf[..n]).map_err(|e| {
                warn!("Failed to write to the output: {e}");
                e
            })?;
            output.flush().map_err(|e| {
                warn!("Failed to flush the output: {e}");
                e
            })?;
        }

        debug!("Streamed stream to output: {bytes} bytes");

        Ok(())
    }
}
