use crate::io::{self, AsyncWrite};

use wasi::http::types::OutgoingBody;

use super::{response::IncomingBody, Body, Request, Response, Result};
use crate::runtime::Reactor;

/// An HTTP client.
#[derive(Debug)]
pub struct Client {}

impl Client {
    /// Create a new instance of `Client`
    pub fn new() -> Self {
        Self {}
    }

    /// Send an HTTP request.
    pub async fn send<B: Body>(&self, req: Request<B>) -> Result<Response<IncomingBody>> {
        let (wasi_req, body) = req.into_outgoing();
        let wasi_body = wasi_req.body().unwrap();
        let body_stream = wasi_body.write().unwrap();

        // 1. Start sending the request head
        let res = wasi::http::outgoing_handler::handle(wasi_req, None).unwrap();

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
