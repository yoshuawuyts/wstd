use wasi::sockets::network::Ipv4SocketAddress;
use wasi::sockets::tcp::{ErrorCode, IpAddressFamily, IpSocketAddress, TcpSocket};

use crate::io;
use crate::iter::AsyncIterator;
use crate::runtime::Reactor;
use std::io::ErrorKind;
use std::net::SocketAddr;

use super::TcpStream;

/// A TCP socket server, listening for connections.
#[derive(Debug)]
pub struct TcpListener {
    socket: TcpSocket,
}

impl TcpListener {
    /// Creates a new TcpListener which will be bound to the specified address.
    ///
    /// The returned listener is ready for accepting connections.
    pub async fn bind(addr: &str) -> io::Result<Self> {
        let addr: SocketAddr = addr
            .parse()
            .map_err(|_| io::Error::other("failed to parse string to socket addr"))?;
        let family = match addr {
            SocketAddr::V4(_) => IpAddressFamily::Ipv4,
            SocketAddr::V6(_) => IpAddressFamily::Ipv6,
        };
        let socket =
            wasi::sockets::tcp_create_socket::create_tcp_socket(family).map_err(to_io_err)?;
        let network = wasi::sockets::instance_network::instance_network();

        let local_address = match addr {
            SocketAddr::V4(addr) => {
                let ip = addr.ip().octets();
                let address = (ip[0], ip[1], ip[2], ip[3]);
                let port = addr.port();
                IpSocketAddress::Ipv4(Ipv4SocketAddress { port, address })
            }
            SocketAddr::V6(_) => todo!("IPv6 not yet supported in `wstd::net::TcpListener`"),
        };
        let reactor = Reactor::current();

        socket
            .start_bind(&network, local_address)
            .map_err(to_io_err)?;
        reactor.wait_for(&socket.subscribe()).await;
        socket.finish_bind().map_err(to_io_err)?;

        socket.start_listen().map_err(to_io_err)?;
        reactor.wait_for(&socket.subscribe()).await;
        socket.finish_listen().map_err(to_io_err)?;
        Ok(Self { socket })
    }

    /// Returns the local socket address of this listener.
    // TODO: make this return an actual socket addr
    pub fn local_addr(&self) -> io::Result<String> {
        let addr = self.socket.local_address().map_err(to_io_err)?;
        Ok(format!("{addr:?}"))
    }

    /// Returns an iterator over the connections being received on this listener.
    pub fn incoming(&self) -> Incoming<'_> {
        Incoming { listener: self }
    }
}

/// An iterator that infinitely accepts connections on a TcpListener.
#[derive(Debug)]
pub struct Incoming<'a> {
    listener: &'a TcpListener,
}

impl<'a> AsyncIterator for Incoming<'a> {
    type Item = io::Result<TcpStream>;

    async fn next(&mut self) -> Option<Self::Item> {
        Reactor::current()
            .wait_for(&self.listener.socket.subscribe())
            .await;
        let (socket, input, output) = match self.listener.socket.accept().map_err(to_io_err) {
            Ok(accepted) => accepted,
            Err(err) => return Some(Err(err)),
        };
        Some(Ok(TcpStream {
            socket,
            input,
            output,
        }))
    }
}

pub(super) fn to_io_err(err: ErrorCode) -> io::Error {
    match err {
        wasi::sockets::network::ErrorCode::Unknown => ErrorKind::Other.into(),
        wasi::sockets::network::ErrorCode::AccessDenied => ErrorKind::PermissionDenied.into(),
        wasi::sockets::network::ErrorCode::NotSupported => ErrorKind::Unsupported.into(),
        wasi::sockets::network::ErrorCode::InvalidArgument => ErrorKind::InvalidInput.into(),
        wasi::sockets::network::ErrorCode::OutOfMemory => ErrorKind::OutOfMemory.into(),
        wasi::sockets::network::ErrorCode::Timeout => ErrorKind::TimedOut.into(),
        wasi::sockets::network::ErrorCode::WouldBlock => ErrorKind::WouldBlock.into(),
        wasi::sockets::network::ErrorCode::InvalidState => ErrorKind::InvalidData.into(),
        wasi::sockets::network::ErrorCode::AddressInUse => ErrorKind::AddrInUse.into(),
        wasi::sockets::network::ErrorCode::ConnectionRefused => ErrorKind::ConnectionRefused.into(),
        wasi::sockets::network::ErrorCode::ConnectionReset => ErrorKind::ConnectionReset.into(),
        wasi::sockets::network::ErrorCode::ConnectionAborted => ErrorKind::ConnectionAborted.into(),
        wasi::sockets::network::ErrorCode::ConcurrencyConflict => ErrorKind::AlreadyExists.into(),
        _ => ErrorKind::Other.into(),
    }
}
