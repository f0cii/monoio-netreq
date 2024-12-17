use bytes::Bytes;
use http::{Method, Version};
use monoio_netreq::http::client::MonoioClient;

const BODY: &str = r#"{"data": {"name": "FNS"}}"#;

#[monoio::main(driver = "uring", timer = true)]
async fn main() -> anyhow::Result<()> {
    let client = MonoioClient::builder() // Connection Pool settings will be default
        .enable_https()
        .http2_prior_knowledge()
        .build();

    let res = client
        .make_request()
        .set_method(Method::POST)
        .set_uri("https://httpbin.org/post")
        .set_header("Content-type", "application/json")
        .set_version(Version::HTTP_2)
        .send_body(Bytes::from_static(BODY.as_ref()))
        .await?;

    assert_eq!(res.status(), 200);
    assert_eq!(res.version(), Version::HTTP_2);

    let string_response = String::from_utf8(res.bytes().await.unwrap().to_vec()).unwrap();
    println!("Result: {}", string_response);

    Ok(())
}