use std::collections::HashMap;
use monoio_netreq::client::client::MonoioClient;


#[monoio::test(driver = "legacy")]
async fn build_a_client() {
    let client = MonoioClient::builder().build_http1().build();
    let http_result = client
        .clone()
        .make_request()
        .set_method("GET")
        .set_uri("https://httpbin.org/ip")
        .set_header("Content-Type", "application/json")
        .send()
        .await;

    assert_eq!(false, http_result.is_err());

    let res = http_result.unwrap();
    assert_eq!(200, res.status());
    let json_response = res.json::<HashMap<String, String>>().await;
    println!("Http Response: {:?}", json_response);
}