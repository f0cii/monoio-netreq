use bytes::Bytes;
use monoio_netreq::client::client::MonoioClient;
mod constants;

use constants::*;

#[monoio::test(driver = "legacy")]
async fn http1_client_without_payload() -> anyhow::Result<()> {
    let client = MonoioClient::builder().build_http1().build();
    let http_result = client
        .clone()
        .make_request()
        .set_method("GET")
        .set_uri("https://httpbin.org/ip")
        .set_header("Content-Type", "application/json")
        .set_version(http::Version::HTTP_11)
        .send()
        .await?;

    let res = http_result;
    assert_eq!(res.status(), 200);
    assert_eq!(res.version(), http::Version::HTTP_11);

    Ok(())
}



#[monoio::test(driver = "legacy")]
async fn http1_client_with_payload() -> anyhow::Result<()> {
    let client = MonoioClient::builder().build_http1().build();
    let url = format!("https://fcm.googleapis.com/v1/projects/{}/messages:send", ID);
    let body = Bytes::from_static(BODY.as_ref());
    let http_result = client
        .clone()
        .make_request()
        .set_method("POST")
        .set_uri(url.clone())
        .set_header("Content-type", "application/json")
        .set_header("Authorization", TOKEN)
        .set_version(http::Version::HTTP_11)
        .send_body(body.clone())
        .await?;

    let res = http_result;
    assert_eq!(res.status(), 200);
    assert_eq!(res.version(), http::Version::HTTP_11);

    Ok(())
}