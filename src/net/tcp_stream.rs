use std::io::Error;

use wasi::{
    io::streams::StreamError,
    sockets::tcp::{InputStream, OutputStream, TcpSocket},
};

use crate::{
    io::{self, AsyncRead, AsyncWrite},
    runtime::Reactor,
};

/// A TCP stream between a local and a remote socket.
pub struct TcpStream<'a> {
    pub(super) reactor: &'a Reactor,
    pub(super) input: InputStream,
    pub(super) output: OutputStream,
    pub(super) socket: TcpSocket,
}

impl<'a> TcpStream<'a> {
    /// Returns the socket address of the remote peer of this TCP connection.
    pub fn peer_addr(&self) -> io::Result<String> {
        let addr = self
            .socket
            .remote_address()
            .map_err(super::tcp_listener::to_io_err)?;
        Ok(format!("{addr:?}"))
    }
}

impl<'a> AsyncRead for TcpStream<'a> {
    async fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reactor.wait_for(self.input.subscribe()).await;
        let slice = match self.input.read(buf.len() as u64) {
            Ok(slice) => slice,
            Err(StreamError::Closed) => return Ok(0),
            Err(StreamError::LastOperationFailed(err)) => {
                return Err(Error::other(err.to_debug_string()));
            }
        };
        let bytes_read = slice.len();
        buf[..bytes_read].copy_from_slice(&slice);
        Ok(bytes_read)
    }
}

impl<'a> AsyncRead for &TcpStream<'a> {
    async fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reactor.wait_for(self.input.subscribe()).await;
        let slice = match self.input.read(buf.len() as u64) {
            Ok(slice) => slice,
            Err(StreamError::Closed) => return Ok(0),
            Err(StreamError::LastOperationFailed(err)) => {
                return Err(Error::other(err.to_debug_string()));
            }
        };
        let bytes_read = slice.len();
        buf[..bytes_read].copy_from_slice(&slice);
        Ok(bytes_read)
    }
}

impl<'a> AsyncWrite for TcpStream<'a> {
    async fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.reactor.wait_for(self.output.subscribe()).await;
        self.output.write(buf).map_err(to_io_err)?;
        Ok(buf.len())
    }

    async fn flush(&mut self) -> io::Result<()> {
        self.output.flush().map_err(to_io_err)
    }
}

impl<'a> AsyncWrite for &TcpStream<'a> {
    async fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.reactor.wait_for(self.output.subscribe()).await;
        self.output.write(buf).map_err(to_io_err)?;
        Ok(buf.len())
    }

    async fn flush(&mut self) -> io::Result<()> {
        self.output.flush().map_err(to_io_err)
    }
}

fn to_io_err(err: StreamError) -> std::io::Error {
    match err {
        StreamError::LastOperationFailed(err) => Error::other(err.to_debug_string()),
        StreamError::Closed => Error::other("Stream was closed"),
    }
}
