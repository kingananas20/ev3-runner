use std::io::{self, Read, Write};

/// Stream framing protocol for continuous streaming
pub struct StreamFramer;

impl StreamFramer {
    /// Create a streaming reader that yields data as it arrives
    pub fn streaming_reader<R: Read>(reader: R) -> ChunkedReader<R> {
        ChunkedReader {
            inner: reader,
            buffer: Vec::new(),
            done: false,
        }
    }

    /// Create a streaming writer that sends data in chunks
    pub fn streaming_writer<W: Write>(writer: W) -> ChunkedWriter<W> {
        ChunkedWriter {
            inner: writer,
            buffer: Vec::new(),
            chunk_size: 8 * 1024,
        }
    }
}

/// Reader adapter that reads chunked protocol
pub struct ChunkedReader<R: Read> {
    inner: R,
    buffer: Vec<u8>,
    done: bool,
}

impl<R: Read> Read for ChunkedReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.done {
            return Ok(0);
        }

        if !self.buffer.is_empty() {
            let to_copy = buf.len().min(self.buffer.len());
            buf[..to_copy].copy_from_slice(&self.buffer[..to_copy]);
            self.buffer.drain(..to_copy);
            return Ok(to_copy);
        }

        let mut len_buf = [0u8; 4];
        self.inner.read_exact(&mut len_buf)?;
        let chunk_len = u32::from_le_bytes(len_buf) as usize;

        if chunk_len == 0 {
            self.done = true;
            return Ok(0);
        }

        if chunk_len <= buf.len() {
            self.inner.read_exact(&mut buf[..chunk_len])?;
            Ok(chunk_len)
        } else {
            self.buffer.resize(chunk_len, 0);
            self.inner.read_exact(&mut self.buffer)?;

            let to_copy = buf.len();
            buf.copy_from_slice(&self.buffer[..to_copy]);
            self.buffer.drain(..to_copy);
            Ok(to_copy)
        }
    }
}

/// Writer adapter that writes data using chunked protocol
pub struct ChunkedWriter<W: Write> {
    inner: W,
    buffer: Vec<u8>,
    chunk_size: usize,
}

impl<W: Write> Write for ChunkedWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.extend_from_slice(buf);

        while self.buffer.len() >= self.chunk_size {
            self.inner
                .write_all(&(self.chunk_size as u32).to_le_bytes())?;
            self.inner.write_all(&self.buffer[..self.chunk_size])?;
            self.buffer.drain(..self.chunk_size);
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        if !self.buffer.is_empty() {
            self.inner
                .write_all(&(self.buffer.len() as u32).to_le_bytes())?;
            self.inner.write_all(&self.buffer)?;
            self.buffer.clear();
        }

        self.inner.flush()
    }
}

impl<W: Write> Drop for ChunkedWriter<W> {
    fn drop(&mut self) {
        let _ = self.flush();
        let _ = self.inner.write_all(&0u32.to_le_bytes());
        let _ = self.inner.flush();
    }
}
