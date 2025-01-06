use super::{fields::header_map_to_wasi, method::to_wasi_method, Error, Result};
use wasi::http::outgoing_handler::OutgoingRequest;
use wasi::http::types::Scheme;

pub use http::Request;

pub(crate) fn try_into_outgoing<T>(request: Request<T>) -> Result<(OutgoingRequest, T)> {
    let wasi_req = OutgoingRequest::new(header_map_to_wasi(request.headers())?);

    let (parts, body) = request.into_parts();

    // Set the HTTP method
    let method = to_wasi_method(parts.method);
    wasi_req
        .set_method(&method)
        .map_err(|()| Error::other(format!("method rejected by wasi-http: {method:?}",)))?;

    // Set the url scheme
    let scheme = match parts.uri.scheme().map(|s| s.as_str()) {
        Some("http") => Scheme::Http,
        Some("https") | None => Scheme::Https,
        Some(other) => Scheme::Other(other.to_owned()),
    };
    wasi_req
        .set_scheme(Some(&scheme))
        .map_err(|()| Error::other(format!("scheme rejected by wasi-http: {scheme:?}")))?;

    // Set authority
    let authority = parts.uri.authority().map(|a| a.as_str());
    wasi_req
        .set_authority(authority)
        .map_err(|()| Error::other(format!("authority rejected by wasi-http {authority:?}")))?;

    // Set the url path + query string
    if let Some(p_and_q) = parts.uri.path_and_query() {
        wasi_req
            .set_path_with_query(Some(&p_and_q.to_string()))
            .map_err(|()| {
                Error::other(format!("path and query rejected by wasi-http {p_and_q:?}"))
            })?;
    }

    // All done; request is ready for send-off
    Ok((wasi_req, body))
}
