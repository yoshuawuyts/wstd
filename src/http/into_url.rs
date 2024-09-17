use url::Url;

use super::Result;

/// A trait for converting types into a `Url`.
///
/// This trait is "sealed" to limit implementations
/// to this crate.
pub trait IntoUrl: IntoUrlSealed {}

impl IntoUrl for Url {}
impl IntoUrl for String {}
impl<'a> IntoUrl for &'a str {}
impl<'a> IntoUrl for &'a String {}

pub trait IntoUrlSealed {
    fn into_url(self) -> Result<Url>;
    fn as_str(&self) -> &str;
}

impl IntoUrlSealed for Url {
    fn into_url(self) -> Result<Url> {
        Ok(self)
    }

    fn as_str(&self) -> &str {
        self.as_ref()
    }
}

impl<'a> IntoUrlSealed for &'a str {
    fn into_url(self) -> Result<Url> {
        Url::parse(self)
            .or(Err(wasi::http::types::ErrorCode::HttpRequestUriInvalid))?
            .into_url()
    }

    fn as_str(&self) -> &str {
        self
    }
}

impl<'a> IntoUrlSealed for &'a String {
    fn into_url(self) -> Result<Url> {
        (&**self).into_url()
    }

    fn as_str(&self) -> &str {
        self.as_ref()
    }
}

impl IntoUrlSealed for String {
    fn into_url(self) -> Result<Url> {
        (&*self).into_url()
    }

    fn as_str(&self) -> &str {
        self.as_ref()
    }
}
