/// The `http` error type.
pub type Error = wasi::http::types::ErrorCode;

/// The `http` result type.
pub type Result<T> = std::result::Result<T, Error>;
