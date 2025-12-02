use super::{Transport, TransportError, stream_framer::StreamFramer};
use std::{
    io::{self, BufReader, BufWriter, Read, Write},
    time::Instant,
};
use tracing::debug;
use zstd::{Decoder, Encoder};

impl Transport {
    pub const FILE_TRANSFER_BUFFER: usize = 512 * 1024;
    const ENCODER_LEVEL: i32 = 1;

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
            let mut enocder = Encoder::new(&mut writer, Self::ENCODER_LEVEL)?;
            let bytes = io::copy(file, &mut enocder);
            enocder.flush()?;
            bytes
        } else {
            io::copy(file, &mut writer)
        }?;
        writer.flush()?;
        drop(writer);
        buf_writer.flush()?;

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
            let mut decoder = Decoder::new(&mut reader)?;
            io::copy(&mut decoder, file)
        } else {
            io::copy(&mut reader, file)
        }?;
        file.flush()?;

        debug!(
            "Receiving file: {bytes} bytes, took {:?}",
            instant.elapsed()
        );

        Ok(())
    }
}
