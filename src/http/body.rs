//! HTTP body types

use crate::io::{AsyncInputStream, AsyncRead, Cursor, Empty};
use core::fmt;
use wasi::http::types::IncomingBody as WasiIncomingBody;

pub use super::{
    error::{Error, ErrorVariant},
    HeaderMap,
};

#[derive(Debug)]
pub(crate) enum BodyKind {
    Fixed(u64),
    Chunked,
}

impl BodyKind {
    pub(crate) fn from_headers(headers: &HeaderMap) -> Result<BodyKind, InvalidContentLength> {
        if let Some(value) = headers.get("content-length") {
            let content_length = std::str::from_utf8(value.as_ref())
                .unwrap()
                .parse::<u64>()
                .map_err(|_| InvalidContentLength)?;
            Ok(BodyKind::Fixed(content_length))
        } else if headers.contains_key("transfer-encoding") {
            Ok(BodyKind::Chunked)
        } else {
            Ok(BodyKind::Chunked)
        }
    }
}

/// A trait representing an HTTP body.
#[doc(hidden)]
pub trait Body: AsyncRead {
    /// Returns the exact remaining length of the iterator, if known.
    fn len(&self) -> Option<usize>;

    /// Returns `true` if the body is known to be empty.
    fn is_empty(&self) -> bool {
        matches!(self.len(), Some(0))
    }
}

/// Conversion into a `Body`.
#[doc(hidden)]
pub trait IntoBody {
    /// What type of `Body` are we turning this into?
    type IntoBody: Body;
    /// Convert into `Body`.
    fn into_body(self) -> Self::IntoBody;
}
impl<T> IntoBody for T
where
    T: Body,
{
    type IntoBody = T;
    fn into_body(self) -> Self::IntoBody {
        self
    }
}

impl IntoBody for String {
    type IntoBody = BoundedBody<Vec<u8>>;
    fn into_body(self) -> Self::IntoBody {
        BoundedBody(Cursor::new(self.into_bytes()))
    }
}

impl IntoBody for &str {
    type IntoBody = BoundedBody<Vec<u8>>;
    fn into_body(self) -> Self::IntoBody {
        BoundedBody(Cursor::new(self.to_owned().into_bytes()))
    }
}

impl IntoBody for Vec<u8> {
    type IntoBody = BoundedBody<Vec<u8>>;
    fn into_body(self) -> Self::IntoBody {
        BoundedBody(Cursor::new(self))
    }
}

impl IntoBody for &[u8] {
    type IntoBody = BoundedBody<Vec<u8>>;
    fn into_body(self) -> Self::IntoBody {
        BoundedBody(Cursor::new(self.to_owned()))
    }
}

/// An HTTP body with a known length
#[derive(Debug)]
pub struct BoundedBody<T>(Cursor<T>);

impl<T: AsRef<[u8]>> AsyncRead for BoundedBody<T> {
    async fn read(&mut self, buf: &mut [u8]) -> crate::io::Result<usize> {
        self.0.read(buf).await
    }
}
impl<T: AsRef<[u8]>> Body for BoundedBody<T> {
    fn len(&self) -> Option<usize> {
        Some(self.0.get_ref().as_ref().len())
    }
}

impl Body for Empty {
    fn len(&self) -> Option<usize> {
        Some(0)
    }
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

impl IncomingBody {
    pub(crate) fn new(
        kind: BodyKind,
        body_stream: AsyncInputStream,
        incoming_body: WasiIncomingBody,
    ) -> Self {
        Self {
            kind,
            body_stream,
            _incoming_body: incoming_body,
        }
    }
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

#[derive(Debug)]
pub struct InvalidContentLength;

impl fmt::Display for InvalidContentLength {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "incoming content-length should be a u64; violates HTTP/1.1".fmt(f)
    }
}

impl std::error::Error for InvalidContentLength {}

impl From<InvalidContentLength> for Error {
    fn from(e: InvalidContentLength) -> Self {
        // TODO: What's the right error code here?
        ErrorVariant::Other(e.to_string()).into()
    }
}
