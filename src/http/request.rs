use crate::io::{empty, Empty};

use super::{header_map_to_wasi, Body, Error, HeaderMap, IntoBody, Method, Result};
use http::uri::Uri;
use wasi::http::outgoing_handler::OutgoingRequest;
use wasi::http::types::Scheme;

/// An HTTP request
#[derive(Debug)]
pub struct Request<B: Body> {
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: B,
}

impl Request<Empty> {
    /// Create a new HTTP request to send off to the client.
    pub fn new(method: Method, uri: Uri) -> Self {
        Self {
            body: empty(),
            method,
            uri,
            headers: HeaderMap::new(),
        }
    }
}

impl<B: Body> Request<B> {
    /// Get the HTTP headers from the impl
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Mutably get the HTTP headers from the impl
    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.headers
    }

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

    pub(crate) fn into_outgoing(self) -> Result<(OutgoingRequest, B)> {
        let wasi_req = OutgoingRequest::new(header_map_to_wasi(&self.headers)?);

        // Set the HTTP method
        let method = self.method.into();
        wasi_req
            .set_method(&method)
            .map_err(|()| Error::other(format!("method rejected by wasi-http: {method:?}",)))?;

        // Set the url scheme
        let scheme = match self.uri.scheme().map(|s| s.as_str()) {
            Some("http") => Scheme::Http,
            Some("https") | None => Scheme::Https,
            Some(other) => Scheme::Other(other.to_owned()),
        };
        wasi_req
            .set_scheme(Some(&scheme))
            .map_err(|()| Error::other(format!("scheme rejected by wasi-http: {scheme:?}")))?;

        // Set authority
        let authority = self.uri.authority().map(|a| a.as_str());
        wasi_req
            .set_authority(authority)
            .map_err(|()| Error::other(format!("authority rejected by wasi-http {authority:?}")))?;

        // Set the url path + query string
        if let Some(p_and_q) = self.uri.path_and_query() {
            wasi_req
                .set_path_with_query(Some(&p_and_q.to_string()))
                .map_err(|()| {
                    Error::other(format!("path and query rejected by wasi-http {p_and_q:?}"))
                })?;
        }

        // All done; request is ready for send-off
        Ok((wasi_req, self.body))
    }
}
