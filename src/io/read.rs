use wasi::io::streams::{InputStream, StreamError};

use crate::io;
use crate::runtime::Reactor;

/// Stream 2kb chunks at a time
const CHUNK_SIZE: u64 = 2048;

/// Read bytes from a source.
pub trait AsyncRead {
    async fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>;
    async fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize>;
}

pub(crate) async fn read_to_end(
    reactor: &Reactor,
    stream: &InputStream,
    buf: &mut Vec<u8>,
) -> io::Result<usize> {
    let starting_len = buf.len();

    loop {
        // Wait for an event to be ready
        let pollable = stream.subscribe();
        reactor.wait_for(pollable).await;

        match stream.read(CHUNK_SIZE) {
            Ok(mut bytes) => buf.append(&mut bytes),
            Err(StreamError::LastOperationFailed(err)) => {
                return Err(std::io::Error::other(err.to_debug_string()));
            }
            Err(StreamError::Closed) => break,
        };
    }

    Ok(buf.len() - starting_len)
}
