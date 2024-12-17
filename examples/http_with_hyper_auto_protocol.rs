use bytes::Bytes;
use http::{Method, Version};
use monoio_netreq::hyper::client::MonoioHyperClient;

const BODY: &str = r#"{"data": {"name": "FNS"}}"#;

#[monoio::main(driver = "uring", timer = true)]
async fn main() -> anyhow::Result<()> {
    let client = MonoioHyperClient::builder()
        .max_idle_connections(5)
        .idle_connection_timeout(5)
        .build(); // https support is not enabled here, notice uring driver in main macro

    let res = client
        .new_request()
        .set_method(Method::POST)
        .set_uri("http://nghttp2.org/httpbin/post")
        .set_header("Content-Type", "application/json")
        .set_version(Version::HTTP_11)
        .send_body(Bytes::from(BODY))
        .await?;

    assert_eq!(res.status(), 200);
    // With auto protocol, the connection gets upgraded to http_2 if the server allows
    assert_eq!(res.version(), Version::HTTP_2);

    let string_response = String::from_utf8(res.raw_body().to_vec()).unwrap();
    println!("Result: {}", string_response);

    Ok(())
}