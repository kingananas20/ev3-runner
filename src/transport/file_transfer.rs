use super::{Transport, TransportError, stream_framer::StreamFramer};
use std::{
    io::{self, BufReader, BufWriter, Read, Write},
    time::Instant,
};
use tracing::{debug, warn};
use zstd::{Decoder, Encoder};

impl Transport {
    pub const FILE_TRANSFER_BUFFER: usize = 512 * 1024;
    const ENCODER_LEVEL: i32 = 3;

    pub fn upload_file<R>(
        &mut self,
        file: &mut R,
        use_compression: bool,
    ) -> Result<(), TransportError>
    where
        R: Read,
    {
        let instant = Instant::now();

        let mut buf_writer = BufWriter::with_capacity(Self::FILE_TRANSFER_BUFFER, &mut self.stream);
        let mut writer = StreamFramer::streaming_writer(&mut buf_writer);

        let bytes = if use_compression {
            let mut enocder = Encoder::new(&mut writer, Self::ENCODER_LEVEL)
                .inspect_err(|e| warn!("Failed to create new zstd encoder: {e}"))?;
            let bytes = io::copy(file, &mut enocder);
            enocder
                .finish()
                .inspect_err(|e| warn!("Failed to finish the zstd encoder: {e}"))?;
            bytes
        } else {
            io::copy(file, &mut writer)
        }
        .inspect_err(|e| warn!("Failed to copy data between the file and the tcp stream: {e}"))?;

        writer
            .flush()
            .inspect_err(|e| warn!("Failed to flush chunkedwriter: {e}"))?;
        drop(writer);
        buf_writer
            .flush()
            .inspect_err(|e| warn!("Failed to flush bufwriter: {e}"))?;

        debug!("Sending file: {bytes} bytes, took {:?}", instant.elapsed());

        Ok(())
    }

    pub fn download_file<W>(
        &mut self,
        file: &mut W,
        use_compression: bool,
    ) -> Result<(), TransportError>
    where
        W: Write,
    {
        let instant = Instant::now();

        let mut reader = StreamFramer::streaming_reader(BufReader::with_capacity(
            Self::FILE_TRANSFER_BUFFER,
            &mut self.stream,
        ));

        let bytes = if use_compression {
            let mut decoder = Decoder::new(&mut reader)
                .inspect_err(|e| warn!("Failed to create new zstd decoder: {e}"))?;
            io::copy(&mut decoder, file)
        } else {
            io::copy(&mut reader, file)
        }
        .inspect_err(|e| warn!("Failed to copy data between the tcp stream and file: {e}"))?;
        file.flush()
            .inspect_err(|e| warn!("Failed to flush the file: {e}"))?;

        debug!(
            "Receiving file: {bytes} bytes, took {:?}",
            instant.elapsed()
        );

        Ok(())
    }
}
