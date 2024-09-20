use wasi::http::types::{IncomingBody as WasiIncomingBody, IncomingResponse};
use wasi::io::streams::{InputStream, StreamError};

use super::{Body, Headers, StatusCode};
use crate::io::AsyncRead;
use crate::runtime::Reactor;

/// An HTTP response
#[derive(Debug)]
pub struct Response<B: Body> {
    headers: Headers,
    status: StatusCode,
    body: B,
}

// #[derive(Debug)]
// enum BodyKind {
//     Fixed(u64),
//     Chunked,
// }

// impl BodyKind {
//     fn from_headers(headers: &Fields) -> BodyKind {
//         dbg!(&headers);
//         if let Some(values) = headers.0.get("content-length") {
//             let value = values
//                 .get(0)
//                 .expect("no value found for content-length; violates HTTP/1.1");
//             let content_length = String::from_utf8(value.to_owned())
//                 .unwrap()
//                 .parse::<u64>()
//                 .expect("content-length should be a u64; violates HTTP/1.1");
//             BodyKind::Fixed(content_length)
//         } else if let Some(values) = headers.0.get("transfer-encoding") {
//             dbg!(values);
//             BodyKind::Chunked
//         } else {
//             dbg!("Encoding neither has a content-length nor transfer-encoding");
//             BodyKind::Chunked
//         }
//     }
// }

impl Response<IncomingBody> {
    pub(crate) fn try_from_incoming_response(
        incoming: IncomingResponse,
        reactor: Reactor,
    ) -> super::Result<Self> {
        let headers: Headers = incoming.headers().into();
        let status = incoming.status().into();

        // `body_stream` is a child of `incoming_body` which means we cannot
        // drop the parent before we drop the child
        let incoming_body = incoming
            .consume()
            .expect("cannot call `consume` twice on incoming response");
        let body_stream = incoming_body
            .stream()
            .expect("cannot call `stream` twice on an incoming body");

        let body = IncomingBody {
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

/// An incoming HTTP body
#[derive(Debug)]
pub struct IncomingBody {
    reactor: Reactor,

    // IMPORTANT: the order of these fields here matters. `incoming_body` must
    // be dropped before `body_stream`.
    body_stream: InputStream,
    _incoming_body: WasiIncomingBody,
}

impl AsyncRead for IncomingBody {
    async fn read(&mut self, buf: &mut [u8]) -> crate::io::Result<usize> {
        // Wait for an event to be ready
        self.reactor.wait_for(self.body_stream.subscribe()).await;

        // Read the bytes from the body stream
        let slice = match self.body_stream.read(buf.len() as u64) {
            Ok(slice) => slice,
            Err(StreamError::Closed) => return Ok(0),
            Err(StreamError::LastOperationFailed(err)) => {
                return Err(std::io::Error::other(err.to_debug_string()));
            }
        };
        let bytes_read = slice.len();
        buf[..bytes_read].copy_from_slice(&slice);
        Ok(bytes_read)
    }
}
