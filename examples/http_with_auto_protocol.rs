use http::{Method, Version};
use monoio_netreq::http::client::MonoioClient;

#[monoio::main(driver = "uring", timer = true)]
async fn main() -> anyhow::Result<()> {
    let client = MonoioClient::builder()
        .max_idle_connections(5)
        .idle_connection_timeout(5)
        .build();

    let res = client
        .make_request()
        .set_method(Method::GET)
        .set_uri("http://nghttp2.org/httpbin/ip")
        .set_header("Content-Type", "application/json")
        .set_version(Version::HTTP_11)
        // .set_version(Version::HTTP_2)
        .send()
        .await?;

    assert_eq!(res.status(), 200);
    assert_eq!(res.version(), Version::HTTP_11);
    // assert_eq!(res.version(), Version::HTTP_2); // Both

    let string_response = String::from_utf8(res.bytes().await.unwrap().to_vec()).unwrap();
    println!("Result: {}", string_response);

    Ok(())
}