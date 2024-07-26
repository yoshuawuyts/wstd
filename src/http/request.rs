use crate::io::{empty, Empty};

use super::{Body, Method};
use url::Url;
use wasi::http::outgoing_handler::OutgoingRequest;
use wasi::http::types::{Headers as WasiHeaders, Scheme};

/// An HTTP request
#[derive(Debug)]
pub struct Request<B: Body> {
    method: Method,
    url: Url,
    headers: WasiHeaders,
    body: B,
}

impl Request<Empty> {
    /// Create a new HTTP request to send off to the client.
    pub fn new(method: Method, url: Url) -> Self {
        Self {
            body: empty(),
            method,
            url,
            headers: WasiHeaders::new(),
        }
    }
}

impl<B: Body> Request<B> {
    /// Set an HTTP body.
    pub fn set_body<C: Body>(self, body: C) -> Request<C> {
        let Self {
            method,
            url,
            headers,
            ..
        } = self;
        Request {
            method,
            url,
            headers,
            body,
        }
    }

    pub fn into_outgoing(self) -> (OutgoingRequest, B) {
        let wasi_req = OutgoingRequest::new(self.headers);

        // Set the HTTP method
        wasi_req.set_method(&self.method.into()).unwrap();

        // Set the url scheme
        let scheme = match self.url.scheme() {
            "http" => Scheme::Http,
            "https" => Scheme::Https,
            other => Scheme::Other(other.to_owned()),
        };
        wasi_req.set_scheme(Some(&scheme)).unwrap();

        // Set the url path + query string
        let path = match self.url.query() {
            Some(query) => format!("{}?{query}", self.url.path()),
            None => self.url.path().to_owned(),
        };
        wasi_req.set_path_with_query(Some(&path)).unwrap();

        // Not sure why we also have to set the authority, but sure we can do
        // that too!
        wasi_req.set_authority(Some(self.url.authority())).unwrap();

        // All done; request is ready for send-off
        (wasi_req, self.body)
    }
}
