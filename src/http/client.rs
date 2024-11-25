use super::{response::IncomingBody, Body, Error, Request, Response, Result};
use crate::io::{self, AsyncWrite};
use crate::runtime::Reactor;
use crate::time::Duration;
use wasi::http::types::{OutgoingBody, RequestOptions as WasiRequestOptions};

/// An HTTP client.
// Empty for now, but permits adding support for RequestOptions soon:
#[derive(Debug)]
pub struct Client {
    options: Option<RequestOptions>,
}

impl Client {
    /// Create a new instance of `Client`
    pub fn new() -> Self {
        Self { options: None }
    }

    /// Send an HTTP request.
    pub async fn send<B: Body>(&self, req: Request<B>) -> Result<Response<IncomingBody>> {
        let (wasi_req, body) = req.into_outgoing()?;
        let wasi_body = wasi_req.body().unwrap();
        let body_stream = wasi_body.write().unwrap();

        // 1. Start sending the request head
        let res = wasi::http::outgoing_handler::handle(wasi_req, self.wasi_options()?).unwrap();

        // 2. Start sending the request body
        io::copy(body, OutputStream::new(body_stream))
            .await
            .expect("io::copy broke oh no");

        // 3. Finish sending the request body
        let trailers = None;
        OutgoingBody::finish(wasi_body, trailers).unwrap();

        // 4. Receive the response
        Reactor::current().wait_for(res.subscribe()).await;
        // NOTE: the first `unwrap` is to ensure readiness, the second `unwrap`
        // is to trap if we try and get the response more than once. The final
        // `?` is to raise the actual error if there is one.
        let res = res.get().unwrap().unwrap()?;
        Ok(Response::try_from_incoming_response(res)?)
    }

    /// Set timeout on connecting to HTTP server
    pub fn set_connect_timeout(&mut self, d: impl Into<Duration>) {
        self.options_mut().connect_timeout = Some(d.into());
    }

    /// Set timeout on recieving first byte of the Response body
    pub fn set_first_byte_timeout(&mut self, d: impl Into<Duration>) {
        self.options_mut().first_byte_timeout = Some(d.into());
    }

    /// Set timeout on recieving subsequent chunks of bytes in the Response body stream
    pub fn set_between_bytes_timeout(&mut self, d: impl Into<Duration>) {
        self.options_mut().between_bytes_timeout = Some(d.into());
    }

    fn options_mut(&mut self) -> &mut RequestOptions {
        match &mut self.options {
            Some(o) => o,
            uninit => {
                *uninit = Some(Default::default());
                uninit.as_mut().unwrap()
            }
        }
    }

    fn wasi_options(&self) -> Result<Option<WasiRequestOptions>> {
        self.options.as_ref().map(|o| o.to_wasi()).transpose()
    }
}

struct OutputStream {
    stream: wasi::http::types::OutputStream,
}

impl OutputStream {
    fn new(stream: wasi::http::types::OutputStream) -> Self {
        Self { stream }
    }
}

impl AsyncWrite for OutputStream {
    async fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let max = self.stream.check_write().unwrap() as usize;
        let max = max.min(buf.len());
        let buf = &buf[0..max];
        self.stream.write(buf).unwrap();
        Reactor::current().wait_for(self.stream.subscribe()).await;
        Ok(max)
    }

    async fn flush(&mut self) -> io::Result<()> {
        self.stream.flush().unwrap();
        Reactor::current().wait_for(self.stream.subscribe()).await;
        Ok(())
    }
}

#[derive(Default, Debug)]
struct RequestOptions {
    connect_timeout: Option<Duration>,
    first_byte_timeout: Option<Duration>,
    between_bytes_timeout: Option<Duration>,
}

impl RequestOptions {
    fn to_wasi(&self) -> Result<WasiRequestOptions> {
        let wasi = WasiRequestOptions::new();
        if let Some(timeout) = self.connect_timeout {
            wasi.set_connect_timeout(Some(*timeout)).map_err(|()| {
                Error::other("wasi-http implementation does not support connect timeout option")
            })?;
        }
        if let Some(timeout) = self.first_byte_timeout {
            wasi.set_first_byte_timeout(Some(*timeout)).map_err(|()| {
                Error::other("wasi-http implementation does not support first byte timeout option")
            })?;
        }
        if let Some(timeout) = self.between_bytes_timeout {
            wasi.set_between_bytes_timeout(Some(*timeout))
                .map_err(|()| {
                    Error::other(
                        "wasi-http implementation does not support between byte timeout option",
                    )
                })?;
        }
        Ok(wasi)
    }
}
