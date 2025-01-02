use wasi::http::types::{IncomingBody as WasiIncomingBody, IncomingResponse};

use super::{fields::header_map_from_wasi, Body, Error, HeaderMap, Result};
use crate::io::{AsyncInputStream, AsyncRead};
use http::StatusCode;

pub use http::Response;

#[derive(Debug)]
enum BodyKind {
    Fixed(u64),
    Chunked,
}

impl BodyKind {
    fn from_headers(headers: &HeaderMap) -> Result<BodyKind> {
        if let Some(value) = headers.get("content-length") {
            let content_length = std::str::from_utf8(value.as_ref())
                .unwrap()
                .parse::<u64>()
                .map_err(|_| {
                    Error::other("incoming content-length should be a u64; violates HTTP/1.1")
                })?;
            Ok(BodyKind::Fixed(content_length))
        } else if headers.contains_key("transfer-encoding") {
            Ok(BodyKind::Chunked)
        } else {
            Ok(BodyKind::Chunked)
        }
    }
}

pub(crate) fn try_from_incoming_response(
    incoming: IncomingResponse,
) -> Result<Response<IncomingBody>> {
    let headers: HeaderMap = header_map_from_wasi(incoming.headers())?;
    // TODO: Does WASI guarantee that the incoming status is valid?
    let status =
        StatusCode::from_u16(incoming.status()).map_err(|err| Error::other(err.to_string()))?;

    let kind = BodyKind::from_headers(&headers)?;
    // `body_stream` is a child of `incoming_body` which means we cannot
    // drop the parent before we drop the child
    let incoming_body = incoming
        .consume()
        .expect("cannot call `consume` twice on incoming response");
    let body_stream = incoming_body
        .stream()
        .expect("cannot call `stream` twice on an incoming body");

    let body = IncomingBody {
        kind,
        body_stream: AsyncInputStream::new(body_stream),
        _incoming_body: incoming_body,
    };

    let mut builder = Response::builder().status(status);

    if let Some(headers_mut) = builder.headers_mut() {
        *headers_mut = headers;
    }

    builder
        .body(body)
        .map_err(|err| Error::other(err.to_string()))
}

/// An incoming HTTP body
#[derive(Debug)]
pub struct IncomingBody {
    kind: BodyKind,
    // IMPORTANT: the order of these fields here matters. `body_stream` must
    // be dropped before `_incoming_body`.
    body_stream: AsyncInputStream,
    _incoming_body: WasiIncomingBody,
}

impl AsyncRead for IncomingBody {
    async fn read(&mut self, out_buf: &mut [u8]) -> crate::io::Result<usize> {
        self.body_stream.read(out_buf).await
    }
}

impl Body for IncomingBody {
    fn len(&self) -> Option<usize> {
        match self.kind {
            BodyKind::Fixed(l) => {
                if l > (usize::MAX as u64) {
                    None
                } else {
                    Some(l as usize)
                }
            }
            BodyKind::Chunked => None,
        }
    }
}
