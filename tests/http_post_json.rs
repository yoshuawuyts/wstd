use serde::{Deserialize, Serialize};
use std::error::Error;
use wstd::http::{request::JsonRequest, Client, Request};

#[derive(Serialize)]
struct TestData {
    test: String,
}

#[derive(Deserialize)]
struct Echo {
    url: String,
}

#[wstd::test]
async fn main() -> Result<(), Box<dyn Error>> {
    let test_data = TestData {
        test: "data".to_string(),
    };
    let request = Request::post("https://postman-echo.com/post").json(&test_data)?;

    let content_type = request
        .headers()
        .get("Content-Type")
        .ok_or_else(|| "request expected to have Content-Type header")?;
    assert_eq!(content_type, "application/json; charset=utf-8");

    let mut response = Client::new().send(request).await?;

    let content_type = response
        .headers()
        .get("Content-Type")
        .ok_or_else(|| "response expected to have Content-Type header")?;
    assert_eq!(content_type, "application/json; charset=utf-8");

    let Echo { url } = response.body_mut().json::<Echo>().await?;
    assert!(
        url.contains("postman-echo.com/post"),
        "expected body url to contain the authority and path, got: {url}"
    );

    Ok(())
}
