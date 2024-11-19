use std::fmt;

/// The `http` result type.
pub type Result<T> = std::result::Result<T, Error>;

/// The `http` error type.
pub struct Error {
    variant: ErrorVariant,
    context: Vec<String>,
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in self.context.iter() {
            write!(f, "in {c}:\n")?;
        }
        match &self.variant {
            ErrorVariant::WasiHttp(e) => write!(f, "wasi http error: {e:?}"),
            ErrorVariant::WasiHeader(e) => write!(f, "wasi header error: {e:?}"),
            ErrorVariant::HeaderName(e) => write!(f, "header name error: {e:?}"),
            ErrorVariant::HeaderValue(e) => write!(f, "header value error: {e:?}"),
            ErrorVariant::Other(e) => write!(f, "{e}"),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.variant {
            ErrorVariant::WasiHttp(e) => write!(f, "wasi http error: {e}"),
            ErrorVariant::WasiHeader(e) => write!(f, "wasi header error: {e}"),
            ErrorVariant::HeaderName(e) => write!(f, "header name error: {e}"),
            ErrorVariant::HeaderValue(e) => write!(f, "header value error: {e}"),
            ErrorVariant::Other(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for Error {}

impl Error {
    pub(crate) fn other(s: impl Into<String>) -> Self {
        ErrorVariant::Other(s.into()).into()
    }
    pub(crate) fn context(self, s: impl Into<String>) -> Self {
        let mut context = self.context;
        context.push(s.into());
        Self {
            variant: self.variant,
            context,
        }
    }
}

impl From<ErrorVariant> for Error {
    fn from(variant: ErrorVariant) -> Error {
        Error {
            variant,
            context: Vec::new(),
        }
    }
}

impl From<wasi::http::types::ErrorCode> for Error {
    fn from(e: wasi::http::types::ErrorCode) -> Error {
        ErrorVariant::WasiHttp(e).into()
    }
}

impl From<wasi::http::types::HeaderError> for Error {
    fn from(e: wasi::http::types::HeaderError) -> Error {
        ErrorVariant::WasiHeader(e).into()
    }
}

impl From<http::header::InvalidHeaderValue> for Error {
    fn from(e: http::header::InvalidHeaderValue) -> Error {
        ErrorVariant::HeaderValue(e).into()
    }
}

impl From<http::header::InvalidHeaderName> for Error {
    fn from(e: http::header::InvalidHeaderName) -> Error {
        ErrorVariant::HeaderName(e).into()
    }
}

#[derive(Debug)]
pub enum ErrorVariant {
    WasiHttp(wasi::http::types::ErrorCode),
    WasiHeader(wasi::http::types::HeaderError),
    HeaderName(http::header::InvalidHeaderName),
    HeaderValue(http::header::InvalidHeaderValue),
    Other(String),
}
