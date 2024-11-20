use wstd::http::{
    error::{ErrorVariant, WasiHttpErrorCode},
    Client, Method, Request,
};

#[wstd::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set first byte timeout to 1/2 second.
    let mut client = Client::new();
    client.set_first_byte_timeout(std::time::Duration::from_millis(500));
    // This get request will connect to the server, which will then wait 1 second before
    // returning a response.
    let request = Request::new(Method::GET, "https://postman-echo.com/delay/1".parse()?);
    let result = client.send(request).await;

    assert!(result.is_err(), "response should be an error");
    let error = result.unwrap_err();
    assert!(
        matches!(
            error.variant(),
            ErrorVariant::WasiHttp(WasiHttpErrorCode::ConnectionReadTimeout)
        ),
        "expected ConnectionReadTimeout error, got: {error:?>}"
    );

    Ok(())
}
