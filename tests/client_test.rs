#[cfg(test)]
mod test {
    #[allow(unused_imports)]
    use bytes::Bytes;
    use http::{Method, Version};
    use monoio_netreq::client::http::MonoioClient;
    #[cfg(feature = "hyper")]
    use monoio_netreq::client::hyper::MonoioHyperClient;

    #[allow(dead_code)]
    const BODY: &str = r#"{"data": {"name": "FNS"}}"#;

    #[monoio::test(driver = "legacy")]
    async fn http1_tls_client() -> anyhow::Result<()> {
        let client = MonoioClient::builder()
            .enable_https()
            .http1_only()
            .build();
        let http_result = client
            .make_request()
            .set_method(Method::GET)
            .set_uri("https://httpbin.org/ip")
            .set_header("Content-Type", "application/json")
            .set_version(Version::HTTP_11)
            .send()
            .await?;

        let res = http_result;
        assert_eq!(res.status(), 200);
        assert_eq!(res.version(), Version::HTTP_11);

        Ok(())
    }

    #[monoio::test(driver = "legacy")]
    async fn http2_tls_client() -> anyhow::Result<()> {
        let client = MonoioClient::builder()
            .enable_https()
            .http2_prior_knowledge()
            .build();
        let url = "https://httpbin.org/post";
        let body = Bytes::from_static(BODY.as_ref());
        let http_result = client
            .make_request()
            .set_method(Method::POST)
            .set_uri(url)
            .set_header("Content-type", "application/json")
            .set_version(Version::HTTP_2)
            .send_body(body.clone())
            .await?;

        let res = http_result;
        assert_eq!(res.status(), 200);
        assert_eq!(res.version(), http::Version::HTTP_2);

        Ok(())
    }

    #[monoio::test(driver = "legacy")]
    // This client sets the Protocol as Auto
    async fn alpn_auto_tls_client() -> anyhow::Result<()> {
        let client = MonoioClient::builder()
            .enable_https()
            .build();
        let http_result = client
            .make_request()
            .set_method(Method::GET)
            .set_uri("https://httpbin.org/ip")
            .set_header("Content-Type", "application/json")
            .send()
            .await?;

        let res = http_result;
        assert_eq!(res.status(), 200);
        assert_eq!(res.version(), Version::HTTP_2);

        Ok(())
    }

    #[monoio::test(driver = "legacy")]
    async fn http1_non_tls_client() -> anyhow::Result<()> {
        let client = MonoioClient::builder()
            .http1_only()
            .build();
        let http_result = client
            .make_request()
            .set_method(Method::GET)
            .set_uri("http://nghttp2.org/httpbin/ip")
            .set_header("Content-Type", "application/json")
            .set_version(Version::HTTP_11)
            .send()
            .await?;

        let res = http_result;
        assert_eq!(res.status(), 200);
        assert_eq!(res.version(), Version::HTTP_11);

        Ok(())
    }

    #[monoio::test(driver = "legacy")]
    async fn http2_non_tls_client() -> anyhow::Result<()> {
        let client = MonoioClient::builder()
            .http2_prior_knowledge()
            .build();
        let body = Bytes::from_static(BODY.as_ref());
        let http_result = client
            .make_request()
            .set_method(Method::POST)
            .set_uri("http://nghttp2.org/httpbin/post")
            .set_header("Content-Type", "application/json")
            .set_version(Version::HTTP_2)
            .send_body(body)
            .await?;

        let res = http_result;
        assert_eq!(res.status(), 200);
        assert_eq!(res.version(), Version::HTTP_2);

        Ok(())
    }

    #[cfg(feature = "hyper")]
    #[monoio::test(driver = "legacy")]
    async fn hyper_http1_non_tls_client() -> anyhow::Result<()> {
        let client = MonoioHyperClient::builder()
            .http1_only()
            .build();
        let body = Bytes::from(BODY);
        let http_result = client
            .new_request()
            .set_method(Method::GET)
            .set_uri("http://nghttp2.org/httpbin/ip")
            .set_header("Content-Type", "application/json")
            .set_version(Version::HTTP_11)
            .send(body)
            .await?;

        assert_eq!(http_result.status(), 200);
        assert_eq!(http_result.version(), Version::HTTP_11);

        Ok(())
    }

    #[cfg(feature = "hyper")]
    #[monoio::test(driver = "legacy")]
    async fn hyper_http2_non_tls_client() -> anyhow::Result<()> {
        let client = MonoioHyperClient::builder()
            .http2_prior_knowledge()
            .build();
        let body = Bytes::from(BODY);
        let http_result = client
            .new_request()
            .set_method(Method::GET)
            .set_uri("http://nghttp2.org/httpbin/ip")
            .set_header("Content-Type", "application/json")
            .set_version(Version::HTTP_2)
            .send(body)
            .await?;

        assert_eq!(http_result.status(), 200);
        assert_eq!(http_result.version(), Version::HTTP_2);

        Ok(())
    }

    #[cfg(feature = "hyper")]
    #[monoio::test(driver = "legacy")]
    async fn hyper_non_tls_client() -> anyhow::Result<()> {
        let client = MonoioHyperClient::builder()
            .build();
        let body = Bytes::from(BODY);
        let http_result = client
            .new_request()
            .set_method(Method::GET)
            .set_uri("http://nghttp2.org/httpbin/ip")
            .set_header("Content-Type", "application/json")
            .set_version(Version::HTTP_11)
            .send(body)
            .await?;

        assert_eq!(http_result.status(), 200);
        // Upgrades the connection to http2
        assert_eq!(http_result.version(), Version::HTTP_2);

        Ok(())
    }
}