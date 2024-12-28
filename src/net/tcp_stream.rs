use std::cell::RefCell;

use wasi::{
    io::streams::{InputStream, OutputStream},
    sockets::tcp::TcpSocket,
};

use crate::io::{self, AsyncInputStream, AsyncOutputStream, AsyncRead, AsyncWrite};

/// A TCP stream between a local and a remote socket.
pub struct TcpStream {
    input: RefCell<AsyncInputStream>,
    output: RefCell<AsyncOutputStream>,
    socket: TcpSocket,
}

impl TcpStream {
    pub(crate) fn new(input: InputStream, output: OutputStream, socket: TcpSocket) -> Self {
        TcpStream {
            input: RefCell::new(AsyncInputStream::new(input)),
            output: RefCell::new(AsyncOutputStream::new(output)),
            socket,
        }
    }
    /// Returns the socket address of the remote peer of this TCP connection.
    pub fn peer_addr(&self) -> io::Result<String> {
        let addr = self
            .socket
            .remote_address()
            .map_err(super::tcp_listener::to_io_err)?;
        Ok(format!("{addr:?}"))
    }

    pub fn split(&self) -> (ReadHalf<'_>, WriteHalf<'_>) {
        (ReadHalf(self), WriteHalf(self))
    }
}

impl Drop for TcpStream {
    fn drop(&mut self) {
        let _ = self.socket.shutdown(wasi::sockets::tcp::ShutdownType::Both);
    }
}

impl AsyncRead for TcpStream {
    async fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.input.borrow_mut().read(buf).await
    }
}

impl AsyncRead for &TcpStream {
    async fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.input.borrow_mut().read(buf).await
    }
}

impl AsyncWrite for TcpStream {
    async fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.output.borrow_mut().write(buf).await
    }

    async fn flush(&mut self) -> io::Result<()> {
        self.output.borrow_mut().flush().await
    }
}

impl AsyncWrite for &TcpStream {
    async fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.output.borrow_mut().write(buf).await
    }

    async fn flush(&mut self) -> io::Result<()> {
        self.output.borrow_mut().flush().await
    }
}

pub struct ReadHalf<'a>(&'a TcpStream);
impl<'a> AsyncRead for ReadHalf<'a> {
    async fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf).await
    }
}

impl<'a> Drop for ReadHalf<'a> {
    fn drop(&mut self) {
        let _ = self
            .0
            .socket
            .shutdown(wasi::sockets::tcp::ShutdownType::Receive);
    }
}

pub struct WriteHalf<'a>(&'a TcpStream);
impl<'a> AsyncWrite for WriteHalf<'a> {
    async fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf).await
    }

    async fn flush(&mut self) -> io::Result<()> {
        self.0.flush().await
    }
}

impl<'a> Drop for WriteHalf<'a> {
    fn drop(&mut self) {
        let _ = self
            .0
            .socket
            .shutdown(wasi::sockets::tcp::ShutdownType::Send);
    }
}
