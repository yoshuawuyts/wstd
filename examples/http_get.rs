use std::error::Error;
use wstd::http::{Client, Method, Request, Url};
use wstd::io::AsyncRead;
use wstd::runtime::block_on;

fn main() -> Result<(), Box<dyn Error>> {
    block_on(async move {
        let request = Request::new(Method::Get, Url::parse("https://postman-echo.com/get")?);
        let mut response = Client::new().send(request).await?;

        let content_type = response
            .headers()
            .get(&"content-type".into())
            .ok_or_else(|| "response expected to have content-type header")?;
        assert_eq!(content_type.len(), 1, "one header value for content-type");
        assert_eq!(content_type[0], b"application/json; charset=utf-8");

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
    })
}
