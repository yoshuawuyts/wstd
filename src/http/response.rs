use wasi::http::types::{IncomingBody as WasiIncomingBody, IncomingResponse};
use wasi::io::streams::{InputStream, StreamError};

use super::{Body, Headers, StatusCode};
use crate::io::AsyncRead;
use crate::iter::AsyncIterator;
use crate::runtime::Reactor;

/// Stream 2kb chunks at a time
const CHUNK_SIZE: u64 = 2048;

/// An HTTP response
#[derive(Debug)]
pub struct Response<B: Body> {
    headers: Headers,
    status: StatusCode,
    body: B,
}

impl Response<IncomingBody> {
    pub(crate) fn try_from_incoming(
        incoming: IncomingResponse,
        reactor: Reactor,
    ) -> super::Result<Self> {
        let headers: Headers = incoming.headers().into();
        let status = incoming.status().into();
        let (_, content_length) = headers
            .0
            .iter()
            .find(|(k, _)| k.to_lowercase() == "content-length")
            .expect("no content-length found; violates HTTP/1.1");
        let content_length = content_length
            .get(0)
            .expect("no value found for content-length; violates HTTP/1.1");
        let content_length = String::from_utf8(content_length.clone())
            .unwrap()
            .parse::<u64>()
            .unwrap();

        // `body_stream` is a child of `incoming_body` which means we cannot
        // drop the parent before we drop the child
        let incoming_body = incoming
            .consume()
            .expect("cannot call `consume` twice on incoming response");
        let body_stream = incoming_body
            .stream()
            .expect("cannot call `stream` twice on an incoming body");

        let body = IncomingBody {
            bytes_read: 0,
            content_length,
            reactor,
            body_stream,
            _incoming_body: incoming_body,
        };

        Ok(Self {
            headers,
            body,
            status,
        })
    }
}

impl<B: Body> Response<B> {
    // Get the HTTP status code
    pub fn status_code(&self) -> StatusCode {
        self.status
    }

    /// Get the HTTP headers from the impl
    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    /// Mutably get the HTTP headers from the impl
    pub fn headers_mut(&mut self) -> &mut Headers {
        &mut self.headers
    }

    pub fn body(&mut self) -> &mut B {
        &mut self.body
    }
}

#[derive(Debug)]
pub struct IncomingBody {
    bytes_read: u64,
    content_length: u64,
    reactor: Reactor,

    // IMPORTANT: the order of these fields here matters. `incoming_body` must
    // be dropped before `body_stream`.
    body_stream: InputStream,
    _incoming_body: WasiIncomingBody,
}

impl AsyncRead for IncomingBody {
    async fn read(&mut self, buf: &mut [u8]) -> crate::io::Result<usize> {
        todo!()
    }
}

impl AsyncIterator for IncomingBody {
    type Item = Result<Vec<u8>, StreamError>;

    async fn next(&mut self) -> Option<Self::Item> {
        // Calculate how many bytes we can read
        let remaining = self.content_length - self.bytes_read;
        let len = remaining.min(CHUNK_SIZE);
        if len == 0 {
            return None;
        }

        // Wait for an event to be ready
        let pollable = self.body_stream.subscribe();
        self.reactor.wait_for(pollable).await;

        // Read the bytes from the body stream
        let buf = self.body_stream.read(len);
        self.bytes_read += len;
        Some(buf)
    }
}
