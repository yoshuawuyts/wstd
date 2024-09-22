use wasi::http::types::{IncomingBody as WasiIncomingBody, IncomingResponse};
use wasi::io::streams::{InputStream, StreamError};

use super::{Body, Headers, StatusCode, Trailers};
use crate::io::AsyncRead;
use crate::runtime::Reactor;

/// An HTTP response
#[derive(Debug)]
pub struct Response<B: Body> {
    headers: Headers,
    status: StatusCode,
    body: B,
}

impl Response<IncomingBody> {
    pub(crate) fn try_from_incoming_response(
        incoming_response: IncomingResponse,
        reactor: Reactor,
    ) -> super::Result<Self> {
        let headers: Headers = incoming_response.headers().into();
        let status = incoming_response.status().into();

        // `body_stream` is a child of `incoming_body` which means we cannot
        // drop the parent before we drop the child,
        // which `incoming_body` is a child of `incoming_response`.
        let incoming_body = incoming_response
            .consume()
            .expect("cannot call `consume` twice on incoming response");
        let body_stream = incoming_body
            .stream()
            .expect("cannot call `stream` twice on an incoming body");

        let content_length = headers
            .get("content-length")
            .and_then(|vals| vals.first())
            .and_then(|v| std::str::from_utf8(v).ok())
            .and_then(|s| s.parse::<u64>().ok());

        let body = IncomingBody {
            content_length,
            reactor,
            trailers: None,
            body_stream: Some(body_stream),
            _incoming_body: Some(incoming_body),
            _incoming_response: Some(incoming_response),
        };

        Ok(Self {
            headers,
            body,
            status,
        })
    }

    ///// NEEDS wasmtime PR to be released:
    ///// https://github.com/bytecodealliance/wasmtime/pull/9208
    //pub fn trailers(&self) -> Option<&Trailers> {
    //    self.body.trailers.as_ref()
    //}
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
    content_length: Option<u64>,
    trailers: Option<Trailers>,

    // IMPORTANT: the order of these fields here matters.
    // Rust drops `struct` fields in the order that they are defined in
    // the source code.
    //
    // `body_stream` must be dropped before `incoming_body`, which
    // must be dropped before `incoming_response`.
    body_stream: Option<InputStream>,
    _incoming_body: Option<WasiIncomingBody>,
    _incoming_response: Option<IncomingResponse>,
}

impl IncomingBody {
    /// Get the full response body as `Vec<u8>`.
    pub async fn bytes(&mut self) -> crate::io::Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(self.content_length.unwrap_or_default() as usize);
        self.read_to_end(&mut buf).await?;
        Ok(buf)
    }
}

impl AsyncRead for IncomingBody {
    async fn read(&mut self, buf: &mut [u8]) -> crate::io::Result<usize> {
        if let Some(stream) = self.body_stream.as_mut() {
            // Wait for an event to be ready
            self.reactor.wait_for(stream.subscribe()).await;

            // Read the bytes from the body stream
            let slice = match stream.read(buf.len() as u64) {
                Ok(slice) => slice,
                Err(StreamError::Closed) => {
                    // stream is done, follow drop order and finalize with trailers

                    // drop `body_stream`
                    let stream = self.body_stream.take();
                    drop(stream);

                    drop(
                        self._incoming_body
                            .take()
                            .expect("IncomingBody is expected to be available"),
                    );

                    // finish `incoming_body` and get trailers
                    // NEEDS wasmtime PR to be released:
                    // https://github.com/bytecodealliance/wasmtime/pull/9208

                    //let incoming_trailers = WasiIncomingBody::finish(
                    //    self._incoming_body
                    //        .take()
                    //        .expect("IncomingBody is expected to be available"),
                    //);
                    //self.reactor.wait_for(incoming_trailers.subscribe()).await;
                    //self.trailers = incoming_trailers
                    //    .get()
                    //    .unwrap() // succeeds since pollable is ready
                    //    .unwrap() // succeeds since first time
                    //    .or(Err(std::io::Error::other("Error receiving trailers")))?
                    //    .map(|trailers| trailers.into());
                    //drop(incoming_trailers);

                    // drop `incoming_response`
                    let incoming_response = self
                        ._incoming_response
                        .take()
                        .expect("IncomingResponse is expected to be available");
                    drop(incoming_response);

                    // clear `content_length`
                    self.content_length = None;

                    return Ok(0);
                }
                Err(StreamError::LastOperationFailed(err)) => {
                    return Err(std::io::Error::other(err.to_debug_string()));
                }
            };
            let bytes_read = slice.len();
            buf[..bytes_read].copy_from_slice(&slice);
            Ok(bytes_read)
        } else {
            // stream is already closed
            Ok(0)
        }
    }
}
