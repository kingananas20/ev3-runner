use super::{Transport, TransportError, stream_framer::StreamFramer};
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

        let writer = BufWriter::with_capacity(512 * 1024, &mut self.stream);
        let mut chunked = StreamFramer::streaming_writer(writer);
        let bytes = io::copy(file, &mut chunked)?;
        chunked.flush()?;

        debug!("Sending file: {bytes} bytes, took {:?}", instant.elapsed());

        Ok(())
    }

    pub fn receive_file<W>(&mut self, file: &mut W) -> Result<(), TransportError>
    where
        W: Write,
    {
        let instant = Instant::now();

        let reader = BufReader::with_capacity(512 * 1024, &mut self.stream);
        let mut chunked = StreamFramer::streaming_reader(reader);
        let bytes = io::copy(&mut chunked, file)?;
        file.flush()?;

        debug!(
            "Receiving file: {bytes} bytes, took {:?}",
            instant.elapsed()
        );

        Ok(())
    }
}
