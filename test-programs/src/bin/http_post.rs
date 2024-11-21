use std::error::Error;
use wstd::http::{Client, HeaderValue, Method, Request};
use wstd::io::AsyncRead;

#[wstd::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut request = Request::new(Method::POST, "https://postman-echo.com/post".parse()?);
    request.headers_mut().insert(
        "content-type",
        HeaderValue::from_str("application/json; charset=utf-8")?,
    );

    let mut response = Client::new()
        .send(request.set_body("{\"test\": \"data\"}"))
        .await?;

    let content_type = response
        .headers()
        .get("Content-Type")
        .ok_or_else(|| "response expected to have Content-Type header")?;
    assert_eq!(content_type, "application/json; charset=utf-8");

    let mut body_buf = Vec::new();
    response.body().read_to_end(&mut body_buf).await?;

    let val: serde_json::Value = serde_json::from_slice(&body_buf)?;
    let body_url = val
        .get("url")
        .ok_or_else(|| "body json has url")?
        .as_str()
        .ok_or_else(|| "body json url is str")?;
    assert!(
        body_url.contains("postman-echo.com/post"),
        "expected body url to contain the authority and path, got: {body_url}"
    );

    let posted_json = val
        .get("json")
        .ok_or_else(|| "body json has 'json' key")?
        .as_object()
        .ok_or_else(|| format!("body json 'json' is object. got {val:?}"))?;

    assert_eq!(posted_json.len(), 1);
    assert_eq!(
        posted_json
            .get("test")
            .ok_or_else(|| "returned json has 'test' key")?
            .as_str()
            .ok_or_else(|| "returned json 'test' key should be str value")?,
        "data"
    );

    Ok(())
}
