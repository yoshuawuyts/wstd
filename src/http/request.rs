use crate::io::{empty, Empty};

use super::{Body, IntoBody, Method};
use http::uri::Uri;
use wasi::http::outgoing_handler::OutgoingRequest;
use wasi::http::types::{Headers as WasiHeaders, Scheme};

/// An HTTP request
#[derive(Debug)]
pub struct Request<B: Body> {
    method: Method,
    uri: Uri,
    headers: WasiHeaders,
    body: B,
}

impl Request<Empty> {
    /// Create a new HTTP request to send off to the client.
    pub fn new(method: Method, uri: Uri) -> Self {
        Self {
            body: empty(),
            method,
            uri,
            headers: WasiHeaders::new(),
        }
    }
}

impl<B: Body> Request<B> {
    /// Set an HTTP body.
    pub fn set_body<C: IntoBody>(self, body: C) -> Request<C::IntoBody> {
        let Self {
            method,
            uri,
            headers,
            ..
        } = self;
        Request {
            method,
            uri,
            headers,
            body: body.into_body(),
        }
    }

    pub(crate) fn into_outgoing(self) -> (OutgoingRequest, B) {
        let wasi_req = OutgoingRequest::new(self.headers);

        // Set the HTTP method
        wasi_req
            .set_method(&self.method.into())
            .expect("method accepted by wasi-http implementation");

        // Set the url scheme
        let scheme = match self.uri.scheme().map(|s| s.as_str()) {
            Some("http") => Scheme::Http,
            Some("https") | None => Scheme::Https,
            Some(other) => Scheme::Other(other.to_owned()),
        };
        wasi_req
            .set_scheme(Some(&scheme))
            .expect("scheme accepted by wasi-http implementation");

        // Set authority
        wasi_req
            .set_authority(self.uri.authority().map(|a| a.as_str()))
            .expect("authority accepted by wasi-http implementation");

        // Set the url path + query string
        if let Some(p_and_q) = self.uri.path_and_query() {
            wasi_req
                .set_path_with_query(Some(&p_and_q.to_string()))
                .expect("path with query accepted by wasi-http implementation")
        }

        // All done; request is ready for send-off
        (wasi_req, self.body)
    }
}
