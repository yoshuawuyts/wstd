pub use http::header::{HeaderMap, HeaderName, HeaderValue};

use super::{Error, Result};
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

pub(crate) fn header_map_to_wasi(header_map: &HeaderMap) -> Result<Fields> {
    let wasi_fields = Fields::new();
    for (key, value) in header_map {
        wasi_fields
            .append(&key.as_str().to_owned(), &value.as_bytes().to_owned())
            .map_err(|e| Error::from(e).context("header named {key}"))?;
    }
    Ok(wasi_fields)
}
