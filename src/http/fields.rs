pub use http::header::{HeaderMap, HeaderName, HeaderValue};
use http::header::{InvalidHeaderName, InvalidHeaderValue};

use super::error::ErrorVariant;
use super::{Error, Result};
use std::fmt;
use wasi::http::types::Fields;

pub(crate) fn header_map_from_wasi(wasi_fields: Fields) -> Result<HeaderMap> {
    let mut output = HeaderMap::new();
    for (key, value) in wasi_fields.entries() {
        let key = HeaderName::from_bytes(key.as_bytes())
            .map_err(|e| Error::from(e).context("header name {key}"))?;
        let value = HeaderValue::from_bytes(&value)
            .map_err(|e| Error::from(e).context("header value for {key}"))?;
        output.append(key, value);
    }
    Ok(output)
}

pub(crate) fn header_map_to_wasi(header_map: &HeaderMap) -> Fields {
    let wasi_fields = Fields::new();
    for (key, value) in header_map {
        // Unwrap because `HeaderMap` has already validated the headers.
        // TODO: Remove the `to_owned()` calls after bytecodealliance/wit-bindgen#1102.
        wasi_fields
            .append(&key.as_str().to_owned(), &value.as_bytes().to_owned())
            .unwrap_or_else(|err| panic!("header named {key}: {err:?}"));
    }
    wasi_fields
}

#[derive(Debug)]
pub(crate) enum InvalidHeader {
    Name(InvalidHeaderName),
    Value(InvalidHeaderValue),
}

impl fmt::Display for InvalidHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Name(e) => e.fmt(f),
            Self::Value(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for InvalidHeader {}

impl From<InvalidHeaderName> for InvalidHeader {
    fn from(e: InvalidHeaderName) -> Self {
        Self::Name(e)
    }
}

impl From<InvalidHeaderValue> for InvalidHeader {
    fn from(e: InvalidHeaderValue) -> Self {
        Self::Value(e)
    }
}

impl From<InvalidHeader> for Error {
    fn from(e: InvalidHeader) -> Self {
        match e {
            InvalidHeader::Name(e) => ErrorVariant::HeaderName(e).into(),
            InvalidHeader::Value(e) => ErrorVariant::HeaderValue(e).into(),
        }
    }
}
