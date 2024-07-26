/// The `wasi-http-client` error type.
pub type Error = wasi::http::types::ErrorCode;

/// The `wasi-http-client` result type.
pub type Result<T> = std::result::Result<T, Error>;
