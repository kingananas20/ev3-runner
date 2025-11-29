use super::{Transport, TransportError};
use std::{
    io::{self, BufReader, BufWriter, Read, Write},
    time::Instant,
};
use tracing::debug;

impl Transport {
    pub fn send_file<R>(&mut self, file: &mut R) -> Result<(), TransportError>
    where
        R: Read,
    {
        let instant = Instant::now();

        let mut writer = BufWriter::with_capacity(512 * 1024, &mut self.stream);
        let bytes = io::copy(file, &mut writer)?;
        writer.flush()?;

        debug!("Sending file: {bytes} bytes, took {:?}", instant.elapsed());

        Ok(())
    }

    pub fn receive_file<W>(&mut self, file: &mut W, size: u64) -> Result<(), TransportError>
    where
        W: Write,
    {
        let instant = Instant::now();

        let reader = BufReader::with_capacity(512 * 1024, &mut self.stream);
        let mut limited = reader.take(size);
        let bytes = io::copy(&mut limited, file)?;
        file.flush()?;

        debug!(
            "Receiving file: {bytes} bytes, took {:?}",
            instant.elapsed()
        );

        Ok(())
    }
}
