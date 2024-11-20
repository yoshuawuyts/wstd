use wasi::http::types::{IncomingBody as WasiIncomingBody, IncomingResponse};
use wasi::io::streams::{InputStream, StreamError};

use super::{fields::header_map_from_wasi, Body, HeaderMap, StatusCode};
use crate::io::AsyncRead;
use crate::runtime::Reactor;

/// Stream 2kb chunks at a time
const CHUNK_SIZE: u64 = 2048;

/// An HTTP response
#[derive(Debug)]
pub struct Response<B: Body> {
    headers: HeaderMap,
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
    pub(crate) fn try_from_incoming_response(incoming: IncomingResponse) -> super::Result<Self> {
        let headers: HeaderMap = header_map_from_wasi(incoming.headers())?;
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
            buf_offset: 0,
            buf: None,
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
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Mutably get the HTTP headers from the impl
    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.headers
    }

    pub fn body(&mut self) -> &mut B {
        &mut self.body
    }
}

/// An incoming HTTP body
#[derive(Debug)]
pub struct IncomingBody {
    buf: Option<Vec<u8>>,
    // How many bytes have we already read from the buf?
    buf_offset: usize,

    // IMPORTANT: the order of these fields here matters. `body_stream` must
    // be dropped before `_incoming_body`.
    body_stream: InputStream,
    _incoming_body: WasiIncomingBody,
}

impl AsyncRead for IncomingBody {
    async fn read(&mut self, out_buf: &mut [u8]) -> crate::io::Result<usize> {
        let buf = match &mut self.buf {
            Some(ref mut buf) => buf,
            None => {
                // Wait for an event to be ready
                let pollable = self.body_stream.subscribe();
                Reactor::current().wait_for(pollable).await;

                // Read the bytes from the body stream
                let buf = match self.body_stream.read(CHUNK_SIZE) {
                    Ok(buf) => buf,
                    Err(StreamError::Closed) => return Ok(0),
                    Err(StreamError::LastOperationFailed(err)) => {
                        return Err(std::io::Error::other(format!(
                            "last operation failed: {}",
                            err.to_debug_string()
                        )))
                    }
                };
                self.buf.insert(buf)
            }
        };

        // copy bytes
        let len = (buf.len() - self.buf_offset).min(out_buf.len());
        let max = self.buf_offset + len;
        let slice = &buf[self.buf_offset..max];
        out_buf[0..len].copy_from_slice(slice);
        self.buf_offset += len;

        // reset the local slice if necessary
        if self.buf_offset == buf.len() {
            self.buf = None;
            self.buf_offset = 0;
        }

        Ok(len)
    }
}
