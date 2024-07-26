//! HTTP support
//!
//! # Example
//!
//! ```rust
//! use wstd::http::{self, Client};
//! use wstd::runtime::block_on;
//!
//! fn main() -> http::Result<()> {
//!     block_on(|reactor| async move {
//!         let resp = Client::new(&reactor)
//!             .post("https://httpbin.org/post")
//!             .send()?;
//!         
//!         println!("status code: {}", resp.status());
//!         Ok(())
//!     })
//! }
//! ```

pub use url::Url;

pub use client::Client;
pub use error::{Error, Result};
pub use fields::{FieldName, FieldValue, Fields, Headers, Trailers};
pub use method::Method;
pub use request::Request;
pub use response::Response;
pub use status_code::StatusCode;

mod client;
mod error;
mod fields;
mod method;
mod request;
mod response;
mod status_code;
