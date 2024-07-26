use wasi::http::types::Method as WasiMethod;

/// The method for the HTTP request
#[derive(Debug)]
#[non_exhaustive]
pub enum Method {
    /// The GET method requests transfer of a current selected representation
    /// for the target resource.
    Get,
    /// The HEAD method is identical to GET except that the server MUST NOT send a message body in
    /// the response.
    Head,
    /// The POST method requests that the target resource process the representation enclosed in
    /// the request according to the resource's own specific semantics.
    Post,
    /// The PUT method requests that the state of the target resource be created or replaced with
    /// the state defined by the representation enclosed in the request message payload.
    Put,
    /// The DELETE method requests that the origin server remove the association between the target
    /// resource and its current functionality.
    Delete,
    /// The CONNECT method requests that the recipient establish a tunnel to the destination origin
    /// server identified by the request-target and, if successful, thereafter restrict its
    /// behavior to blind forwarding of packets, in both directions, until the tunnel is closed.
    Connect,
    /// The OPTIONS method requests information about the communication options available for the
    /// target resource, at either the origin server or an intervening intermediary.
    Options,
    /// The TRACE method requests a remote, application-level loop-back of the request message.
    Trace,
    /// The PATCH method requests that a set of changes described in the request entity be applied
    /// to the resource identified by the Request- URI.
    ///
    Patch,
    /// Send a method not covered by this list.
    Other(String),
}

impl From<Method> for WasiMethod {
    fn from(value: Method) -> Self {
        match value {
            Method::Get => WasiMethod::Get,
            Method::Head => WasiMethod::Head,
            Method::Post => WasiMethod::Post,
            Method::Put => WasiMethod::Put,
            Method::Delete => WasiMethod::Delete,
            Method::Connect => WasiMethod::Connect,
            Method::Options => WasiMethod::Options,
            Method::Trace => WasiMethod::Trace,
            Method::Patch => WasiMethod::Patch,
            Method::Other(s) => WasiMethod::Other(s),
        }
    }
}

impl From<WasiMethod> for Method {
    fn from(value: WasiMethod) -> Self {
        match value {
            WasiMethod::Get => Method::Get,
            WasiMethod::Head => Method::Head,
            WasiMethod::Post => Method::Post,
            WasiMethod::Put => Method::Put,
            WasiMethod::Delete => Method::Delete,
            WasiMethod::Connect => Method::Connect,
            WasiMethod::Options => Method::Options,
            WasiMethod::Trace => Method::Trace,
            WasiMethod::Patch => Method::Patch,
            WasiMethod::Other(s) => Method::Other(s),
        }
    }
}
