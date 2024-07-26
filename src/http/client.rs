use super::{Request, Response, Result};
use crate::runtime::Reactor;

/// An HTTP client.
#[derive(Debug)]
pub struct Client {
    reactor: Reactor,
}

impl Client {
    /// Create a new instance of `Client`
    pub fn new(reactor: Reactor) -> Self {
        Self { reactor }
    }

    /// Send an HTTP request.
    pub async fn send(&self, req: Request) -> Result<Response> {
        let wasi_req = req.into();
        let res = wasi::http::outgoing_handler::handle(wasi_req, None).unwrap();
        self.reactor.wait_for(res.subscribe()).await;

        // NOTE: the first `unwrap` is to ensure readiness, the second `unwrap`
        // is to trap if we try and get the response more than once. The final
        // `?` is go raise the actual error if there is one.
        let res = res.get().unwrap().unwrap()?;
        Ok(Response::try_from_incoming(res, self.reactor.clone())?)
    }
}
