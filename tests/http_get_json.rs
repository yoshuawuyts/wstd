use serde::Deserialize;
use std::error::Error;
use wstd::http::{Client, Request};
use wstd::io::empty;

#[derive(Deserialize)]
struct Echo {
    url: String,
}

#[wstd::test]
async fn main() -> Result<(), Box<dyn Error>> {
    let request = Request::get("https://postman-echo.com/get").body(empty())?;

    let mut response = Client::new().send(request).await?;

    let content_type = response
        .headers()
        .get("Content-Type")
        .ok_or_else(|| "response expected to have Content-Type header")?;
    assert_eq!(content_type, "application/json; charset=utf-8");

    let Echo { url } = response.body_mut().json::<Echo>().await?;
    assert!(
        url.contains("postman-echo.com/get"),
        "expected body url to contain the authority and path, got: {url}"
    );

    Ok(())
}
