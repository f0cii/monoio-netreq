use http::{Method, Version};
use monoio_netreq::hyper::client::MonoioHyperClient;

#[monoio::main(driver = "legacy", timer = true)]
async fn main() -> anyhow::Result<()> {
    let client = MonoioHyperClient::builder()
        .max_idle_connections(5)
        .idle_connection_timeout(5)
        .http1_only()
        .enable_https() // Https support in hyper won't work with uring driver, notice the main macro
        .build();

    let res = client
        .new_request()
        .set_method(Method::GET)
        .set_uri("https://hyper.rs")
        .set_header("Content-Type", "application/json")
        .set_version(Version::HTTP_11)
        .send()
        .await?;

    assert_eq!(res.status(), 200);
    assert_eq!(res.version(), Version::HTTP_11);
    // Https connections are negotiated via alpn, hence there won't be any upgrade here
    // Connections only get upgraded when using http only

    Ok(())
}