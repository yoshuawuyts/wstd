use super::{error::WasiHttpErrorCode, fields::header_map_to_wasi, HeaderMap, Response};
use crate::io::{AsyncOutputStream, AsyncWrite};
use wasi::exports::http::incoming_handler::ResponseOutparam;
use wasi::http::types::OutgoingResponse;

/// This is passed into the [`proxy`] `main` function and holds the state
/// needed for a handler to produce a response, or fail. There are two ways to
/// respond, with [`Responder::start_response`] to stream the body in, or
/// [`Responder::respond`] to give the body as a single string. See those
/// functions for examples.
///
/// [`proxy`]: crate::proxy
#[must_use]
pub struct Responder {
    outparam: ResponseOutparam,
}

impl Responder {
    /// Start responding with the given `Response` and return an `OutgoingBody`
    /// stream to write the body to.
    ///
    /// # Example
    ///
    /// ```
    /// # use wstd::http::{body::IncomingBody, Response, Request};
    /// # use wstd::http::proxy::{BodyForthcoming, Finished, Responder};
    /// # use crate::wstd::io::AsyncWrite;
    /// # async fn example(responder: Responder) -> Finished {
    ///     let mut body = responder.start_response(Response::new(BodyForthcoming));
    ///     let result = body
    ///         .write_all("Hello!\n".as_bytes())
    ///         .await;
    ///     body.finish(result, None)
    /// # }
    /// ```
    pub fn start_response(self, response: Response<BodyForthcoming>) -> OutgoingBody {
        let wasi_headers = header_map_to_wasi(response.headers());
        let wasi_response = OutgoingResponse::new(wasi_headers);
        let wasi_status = response.status().as_u16();

        // Unwrap because `StatusCode` has already validated the status.
        wasi_response.set_status_code(wasi_status).unwrap();

        // Unwrap because we can be sure we only call these once.
        let wasi_body = wasi_response.body().unwrap();
        let wasi_stream = wasi_body.write().unwrap();

        // Tell WASI to start the show.
        ResponseOutparam::set(self.outparam, Ok(wasi_response));

        OutgoingBody {
            stream: AsyncOutputStream::new(wasi_stream),
            body: wasi_body,
        }
    }

    /// Respond with the given `Response` which contains the already-completed
    /// body, and optional trailers.
    ///
    /// A Content-Length header is automatically added.
    ///
    /// # Example
    ///
    /// ```
    /// # use wstd::http::{body::IncomingBody, Response, Request};
    /// # use wstd::http::proxy::{BodyForthcoming, Finished, Responder};
    /// # async fn example(responder: Responder) -> Finished {
    ///     responder
    ///         .respond(Response::new("Hello!\n".as_bytes()), None)
    ///         .await
    /// # }
    /// ```
    // TODO: Should we use something like `IntoBody` instead of `AsRef<[u8]>`?
    pub async fn respond<Body: AsRef<[u8]>>(
        self,
        response: Response<Body>,
        trailers: Option<HeaderMap>,
    ) -> Finished {
        let headers = response.headers();
        let status = response.status().as_u16();

        let wasi_headers = header_map_to_wasi(headers);

        // Consume the `response` and prepare to write the body.
        let body = response.into_body();
        let body = body.as_ref();

        // Automatically add a Content-Length header.
        wasi_headers
            .append(
                &"content-length".to_owned(),
                &body.len().to_string().into_bytes(),
            )
            .unwrap();

        let wasi_response = OutgoingResponse::new(wasi_headers);

        // Unwrap because `StatusCode` has already validated the status.
        wasi_response.set_status_code(status).unwrap();

        // Unwrap because we can be sure we only call these once.
        let wasi_body = wasi_response.body().unwrap();
        let wasi_stream = wasi_body.write().unwrap();

        // Tell WASI to start the show.
        ResponseOutparam::set(self.outparam, Ok(wasi_response));

        let mut outgoing_body = OutgoingBody {
            stream: AsyncOutputStream::new(wasi_stream),
            body: wasi_body,
        };

        let result = outgoing_body.write_all(body).await;
        outgoing_body.finish(result, trailers)
    }

    /// This is used by the `main` macro.
    #[doc(hidden)]
    pub fn new(outparam: ResponseOutparam) -> Self {
        Self { outparam }
    }

    /// This is used by the `main` macro.
    #[doc(hidden)]
    pub fn fail(self, err: WasiHttpErrorCode) -> Finished {
        ResponseOutparam::set(self.outparam, Err(err));
        Finished(())
    }
}

/// A placeholder for use as the type parameter to [`Response`] to indicate
/// that the body has not yet started. This is used with
/// [`Responder::start_response`], which has a `Response<BodyForthcoming>`
/// argument.
///
/// To instead start the response and obtain the output stream for the body,
/// use [`Responder::respond`].
pub struct BodyForthcoming;

/// The output stream for the body, implementing [`AsyncWrite`]. Call
/// [`Responder::start_response`] to obtain one. Once the body is complete,
/// it must be declared finished, using [`OutgoingBody::finish`].
#[must_use]
pub struct OutgoingBody {
    // IMPORTANT: the order of these fields here matters. `stream` must
    // be dropped before `body`.
    stream: AsyncOutputStream,
    body: wasi::http::types::OutgoingBody,
}

impl OutgoingBody {
    /// Finish the body, optionally with trailers, and return a `Finished`
    /// token to be returned from the proxy [`main` function] to indicate that
    /// the response is finished.
    ///
    /// `result` is a `std::io::Result` for reporting any I/O errors that
    /// occur while writing to the body stream.
    ///
    /// [`main` function]: crate::main
    pub fn finish(self, result: std::io::Result<()>, trailers: Option<HeaderMap>) -> Finished {
        // The stream is a child resource of the `OutgoingBody`, so ensure that
        // it's dropped first.
        drop(self.stream);

        if result.is_ok() {
            let wasi_trailers = trailers.map(|trailers| header_map_to_wasi(&trailers));

            wasi::http::types::OutgoingBody::finish(self.body, wasi_trailers)
                .expect("body length did not match Content-Length header value");
        } else {
            // As in `fail`, there's no need to do anything on failure.
            // TODO: Should we log the failure somewhere?
        }

        Finished(())
    }

    /// Return a `Finished` token that can be returned from a handler to
    /// indicate that the body is not finished and should be considered
    /// corrupted.
    pub fn fail(self) -> Finished {
        // No need to do anything; omitting the call to `finish` achieves
        // the desired effect.
        Finished(())
    }

    /// Return a reference to the underlying `AsyncOutputStream`.
    ///
    /// This usually isn't needed, as `OutgoingBody` implements `AsyncWrite`
    /// too, however it is useful for code that expects to work with
    /// `AsyncOutputStream` specifically.
    pub fn stream(&mut self) -> &mut AsyncOutputStream {
        &mut self.stream
    }
}

impl AsyncWrite for OutgoingBody {
    async fn write(&mut self, buf: &[u8]) -> crate::io::Result<usize> {
        self.stream.write(buf).await
    }

    async fn flush(&mut self) -> crate::io::Result<()> {
        self.stream.flush().await
    }
}

impl AsyncWrite for &mut OutgoingBody {
    async fn write(&mut self, buf: &[u8]) -> crate::io::Result<usize> {
        (*self).write(buf).await
    }

    async fn flush(&mut self) -> crate::io::Result<()> {
        (*self).flush().await
    }
}

/// An opaque value returned from a handler indicating that the body is
/// finished, either by [`OutgoingBody::finish`] or [`OutgoingBody::fail`].
#[must_use]
pub struct Finished(());

impl Drop for Finished {
    fn drop(&mut self) {
        unreachable!("`Finished::drop` called; proxy components shouldn't do fallible work after finishing their response");
    }
}
