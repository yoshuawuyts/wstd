use crate::io;

const CHUNK_SIZE: usize = 2048;

/// Read bytes from a source.
pub trait AsyncRead {
    async fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>;
    async fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        // total bytes written to buf
        let mut n = 0;

        loop {
            // grow buf, if less than default chuck size
            if buf.len() < n + CHUNK_SIZE {
                buf.resize(n + CHUNK_SIZE, 0u8);
            }

            let len = self.read(&mut buf[n..]).await?;
            if len == 0 {
                buf.truncate(n);
                return Ok(n);
            }

            n += len;
        }
    }
}
