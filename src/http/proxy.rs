use crate::http::{Body, Request, Response};

/// Implement this trait to implement the methods for a proxy application,
/// which accepts requests and produces responses.
pub trait Proxy<B: Body> {
    /// Handle a single incoming request, producing a response.
    fn handle(request: Request<B>) -> Response<B>;
}

/// Create a proxy application. See examples/proxy.rs for an example.
#[macro_export]
macro_rules! proxy {
    ($ty:ident) => {
        $crate::proxy!($ty with_types_in wasi);
    };
    ($ty:ident with_types_in $($path_to_types_root:tt)*) => {
        struct Wrapper($ty);

        impl wasi::exports::http::incoming_handler::Guest for Wrapper {
            fn handle(
                request: wasi::http::types::IncomingRequest,
                response_out: wasi::http::types::ResponseOutparam
            ) {
                // First convert the request from the WASI type into wstd's public type.
                //
                // TODO: Do we need to do error handling for all these things,
                // or can we assume WASI has done them already, and just unwrap?
                let method = match request.method() {
                    wasi::http::types::Method::Get => $crate::http::Method::GET,
                    wasi::http::types::Method::Head => $crate::http::Method::HEAD,
                    wasi::http::types::Method::Post => $crate::http::Method::POST,
                    wasi::http::types::Method::Put => $crate::http::Method::PUT,
                    wasi::http::types::Method::Delete => $crate::http::Method::DELETE,
                    wasi::http::types::Method::Connect => $crate::http::Method::CONNECT,
                    wasi::http::types::Method::Options => $crate::http::Method::OPTIONS,
                    wasi::http::types::Method::Trace => $crate::http::Method::TRACE,
                    wasi::http::types::Method::Patch => $crate::http::Method::PATCH,
                    wasi::http::types::Method::Other(_) => {
                        wasi::http::types::ResponseOutparam::set(
                            response_out,
                            Err(wasi::http::types::ErrorCode::HttpRequestMethodInvalid)
                        );
                        return;
                    }
                };
                let scheme = match request.scheme() {
                    Some(wasi::http::types::Scheme::Http) => Some($crate::http::Scheme::HTTP),
                    Some(wasi::http::types::Scheme::Https) => Some($crate::http::Scheme::HTTPS),
                    Some(wasi::http::types::Scheme::Other(other)) => {
                        wasi::http::types::ResponseOutparam::set(
                            response_out,
                            Err(wasi::http::types::ErrorCode::HttpRequestUriInvalid)
                        );
                        return;
                    }
                    None => None,
                };
                let authority = match request.authority() {
                    Some(authority) => match $crate::http::Authority::from_maybe_shared(authority) {
                        Ok(authority) => Some(authority),
                        Err(_) => {
                            wasi::http::types::ResponseOutparam::set(
                                response_out,
                                Err(wasi::http::types::ErrorCode::HttpRequestUriInvalid)
                            );
                            return;
                        }
                    }
                    None => None,
                };
                let path_with_query = match request.path_with_query() {
                    Some(path_with_query) => match $crate::http::PathAndQuery::from_maybe_shared(path_with_query) {
                        Ok(path_with_query) => Some(path_with_query),
                        Err(_) => {
                            wasi::http::types::ResponseOutparam::set(
                                response_out,
                                Err(wasi::http::types::ErrorCode::HttpRequestUriInvalid)
                            );
                            return;
                        }
                    }
                    None => None,
                };
                let request = $crate::http::Request::incoming(
                    method,
                    scheme,
                    authority,
                    path_with_query
                );

                // Send the converted request to the user's handle function.
                let response = $ty::handle(request);

                // Convert the response from the public wstd type to the WASI type.
                let headers = wasi::http::types::Fields::new();
                for (name, value) in response.headers() {
                    let name = name.as_str().to_owned();
                    let value = value.as_bytes().to_owned();
                    match headers.append(&name, &value) {
                        Ok(()) => {}
                        Err(err) => {
                            wasi::http::types::ResponseOutparam::set(
                                response_out,
                                Err(wasi::http::types::ErrorCode::InternalError(Some(err.to_string())))
                            );
                            return;
                        }
                    }
                }
                wasi::http::types::ResponseOutparam::set(
                    response_out,
                    Ok(wasi::http::types::OutgoingResponse::new(headers))
                );
            }
        }

        wasi::http::proxy::export!(Wrapper with_types_in $($path_to_types_root)*);
    };
}
