use std::error::Error;
use wstd::http::{Client, Method, Request};
use wstd::io::AsyncRead;

#[wstd::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let request = Request::new(Method::GET, "https://postman-echo.com/get".parse()?);
    let mut response = Client::new().send(request).await?;

    let content_type = response
        .headers()
        .get("Content-Type")
        .ok_or_else(|| "response expected to have Content-Type header")?;
    assert_eq!(content_type, "application/json; charset=utf-8");

    // Would much prefer read_to_end here:
    let mut body_buf = vec![0; 4096];
    let body_len = response.body().read(&mut body_buf).await?;
    body_buf.truncate(body_len);

    let val: serde_json::Value = serde_json::from_slice(&body_buf)?;
    let body_url = val
        .get("url")
        .ok_or_else(|| "body json has url")?
        .as_str()
        .ok_or_else(|| "body json url is str")?;
    assert!(
        body_url.contains("postman-echo.com/get"),
        "expected body url to contain the authority and path, got: {body_url}"
    );

    Ok(())
}
