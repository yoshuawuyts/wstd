#![no_main]

pub use wstd::http::{HeaderMap, Proxy, Request, Response};
pub use wstd::io::{empty, Empty};

struct MyIncomingHandler;

impl Proxy<Empty> for MyIncomingHandler {
    fn handle(_request: Request<Empty>) -> Response<Empty> {
        let hdrs = HeaderMap::new();
        let body = empty();

        Response::new(hdrs, body)
    }
}

wstd::proxy!(MyIncomingHandler);
