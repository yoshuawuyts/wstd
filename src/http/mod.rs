//! HTTP networking support
//!
pub use http::status::StatusCode;
pub use http::uri::Uri;

#[doc(inline)]
pub use body::{Body, IntoBody};
pub use client::Client;
pub use error::{Error, Result};
pub use fields::{HeaderMap, HeaderName, HeaderValue};
pub use method::Method;
pub use request::Request;
pub use response::Response;

pub mod body;

mod client;
pub mod error;
mod fields;
mod method;
mod request;
mod response;
