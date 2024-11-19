pub use http::header::{HeaderMap as Fields, HeaderName as FieldName, HeaderValue as FieldValue};
pub type Headers = Fields;
pub type Trailers = Fields;

use super::{Error, Result};
use wasi::http::types::Fields as WasiFields;

pub(crate) fn fields_from_wasi(wasi_fields: WasiFields) -> Result<Fields> {
    let mut output = Fields::new();
    for (key, value) in wasi_fields.entries() {
        let key = FieldName::from_bytes(key.as_bytes())
            .map_err(|e| Error::from(e).context("header name {key}"))?;
        let value = FieldValue::from_bytes(&value)
            .map_err(|e| Error::from(e).context("header value for {key}"))?;
        output.append(key, value);
    }
    Ok(output)
}

pub(crate) fn fields_to_wasi(fields: &Fields) -> Result<WasiFields> {
    let wasi_fields = WasiFields::new();
    for (key, value) in fields {
        wasi_fields
            .append(&key.as_str().to_owned(), &value.as_bytes().to_owned())
            .map_err(|e| Error::from(e).context("header named {key}"))?;
    }
    Ok(wasi_fields)
}
