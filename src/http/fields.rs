use wasi::http::types::{
    FieldKey as WasiFieldKey, FieldValue as WasiFieldValue, Fields as WasiFields,
};

/// A type alias for [`Fields`] when used as HTTP headers.
pub type Headers = Fields;

/// A type alias for [`Fields`] when used as HTTP trailers.
pub type Trailers = Fields;

/// A type alias for the `field-key` in`wasi:http`.
pub type FieldName = WasiFieldKey;

/// A type alias for the `field-value` in`wasi:http`.
pub type FieldValue = WasiFieldValue;

/// A type alias for the `fields` resource in`wasi:http`.
pub type Fields = WasiFields;
