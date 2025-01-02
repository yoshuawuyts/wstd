use wstd::http::body::IncomingBody;
use wstd::http::proxy::{BodyForthcoming, Finished, Responder};
use wstd::http::{Request, Response};
use wstd::io::{copy, AsyncWrite};

#[wstd::proxy]
async fn main(request: Request<IncomingBody>, responder: Responder) -> Finished {
    match request.uri().path_and_query().unwrap().as_str() {
        "/wait" => http_wait(request, responder).await,
        "/echo" => http_echo(request, responder).await,
        "/" | _ => http_home(request, responder).await,
    }
}

async fn http_home(_request: Request<IncomingBody>, responder: Responder) -> Finished {
    // To send a single string as the response body, use `Responder::respond`.
    responder
        .respond(Response::new(b"Hello, wasi:http/proxy world!\n"), None)
        .await
}

async fn http_wait(_request: Request<IncomingBody>, responder: Responder) -> Finished {
    // Get the time now
    let now = wasi::clocks::monotonic_clock::now();

    // Sleep for 1 second
    let nanos = 1_000_000_000;
    let pollable = wasi::clocks::monotonic_clock::subscribe_duration(nanos);
    pollable.block();

    // Compute how long we slept for.
    let elapsed = wasi::clocks::monotonic_clock::now() - now;
    let elapsed = elapsed / 1_000_000; // change to millis

    // To stream data to the response body, use `Responder::start_response`.
    let mut body = responder.start_response(Response::new(BodyForthcoming));
    let result = body
        .write_all(format!("slept for {elapsed} millis\n").as_bytes())
        .await;
    body.finish(result, None)
}

async fn http_echo(mut request: Request<IncomingBody>, responder: Responder) -> Finished {
    // Stream data from the request body to the response body.
    let mut body = responder.start_response(Response::new(BodyForthcoming));
    let result = copy(request.body_mut(), &mut body).await;
    body.finish(result, None)
}
