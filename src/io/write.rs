use crate::io;

/// Write bytes to a sink.
pub trait AsyncWrite {
    // Required methods
    async fn write(&mut self, buf: &[u8]) -> io::Result<usize>;
    async fn flush(&mut self) -> io::Result<()>;
}
