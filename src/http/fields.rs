pub use http::header::{HeaderMap, HeaderName, HeaderValue};

use super::{Error, error::Context};
use wasip2::http::types::Fields;

pub(crate) fn header_map_from_wasi(wasi_fields: Fields) -> Result<HeaderMap, Error> {
    let mut output = HeaderMap::new();
    for (key, value) in wasi_fields.entries() {
        let key =
            HeaderName::from_bytes(key.as_bytes()).with_context(|| format!("header name {key}"))?;
        let value =
            HeaderValue::from_bytes(&value).with_context(|| format!("header value for {key}"))?;
        output.append(key, value);
    }
    Ok(output)
}

pub(crate) fn header_map_to_wasi(header_map: &HeaderMap) -> Result<Fields, Error> {
    let wasi_fields = Fields::new();
    for (key, value) in header_map {
        let key = key.as_str().to_ascii_lowercase();
        if FORBIDDEN_HEADERS
            .iter()
            .find(|k| k.as_str() == key)
            .is_none()
        {
            wasi_fields
                .append(key.as_str(), value.as_bytes())
                .with_context(|| format!("wasi rejected header `{key}: {value:?}`"))?
        }
    }
    Ok(wasi_fields)
}

// Optimization opportunity: use a trie here
const FORBIDDEN_HEADERS: [HeaderName; 11] = [
    http::header::CONNECTION,
    HeaderName::from_static("keep-alive"),
    http::header::PROXY_AUTHENTICATE,
    http::header::PROXY_AUTHORIZATION,
    HeaderName::from_static("proxy-connection"),
    http::header::TRANSFER_ENCODING,
    http::header::UPGRADE,
    http::header::HOST,
    HeaderName::from_static("http2-settings"),
    http::header::EXPECT,
    http::header::CONTENT_LENGTH,
];
