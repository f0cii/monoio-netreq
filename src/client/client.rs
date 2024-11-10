use std::rc::Rc;
use std::time::Duration;
use http::{Request, Uri};
use monoio::net::TcpStream;
use monoio_http::common::body::HttpBody;
use monoio_transports::http::{HttpConnector};
use monoio_transports::connectors::{Connector, TcpConnector, TcpTlsAddr, TlsConnector, TlsStream};

use super::Proto;
use crate::response::{Response};
use crate::error::{Error, Result};
use crate::request::HttpRequest;

pub struct MonoioClient {
    inner: Rc<ClientInner>,
}

impl MonoioClient {
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }
}


struct ClientInner {
    config: ClientConfig,
    connector: TcpConnector,
    http_connector: HttpConnector<TlsConnector<TcpConnector>, TcpTlsAddr, TlsStream<TcpStream>>,
}


#[derive(Default, Clone)]
struct ClientConfig {
    proto: Proto,
    pool_disabled: bool,
    max_idle_connections: usize,
    idle_timeout_duration: u32,
    read_timeout: Option<Duration>,
    initial_max_streams: Option<usize>,
    max_concurrent_streams: Option<u32>,
}


#[derive(Default)]
pub struct ClientBuilder {
    build_config: ClientConfig
}

impl ClientBuilder {
    pub fn disable_connection_pool(mut self) -> Self {
        self.build_config.pool_disabled = true;
        self
    }

    pub fn set_idle_connections_per_host(mut self, val: usize) -> Self {
        self.build_config.max_idle_connections = val;
        self
    }

    pub fn idle_connection_timeout_duration(mut self, val: u32) -> Self {
        self.build_config.idle_timeout_duration = val;
        self
    }

    pub fn set_read_timeout(mut self, val: u64) -> Self {
        self.build_config.read_timeout = Some(Duration::from_secs(val));
        self
    }

    pub fn initial_max_streams(mut self, val: usize) -> Self {
        self.build_config.initial_max_streams = Some(val);
        self
    }

    pub fn max_concurrent_streams(mut self, val: u32) -> Self {
        self.build_config.max_concurrent_streams = Some(val);
        self
    }

    pub fn build_http1(mut self) -> Self {
        self.build_config.proto = Proto::Http1;
        self
    }

    pub fn build_http2(mut self) -> Self {
        self.build_config.proto = Proto::Http2;
        self
    }

    pub fn build(self) -> MonoioClient {
        let config = self.build_config.clone();

        let connector = TcpConnector::default();
        let alpn = match config.proto {
            Proto::Http1 => vec!["http/1.1"],
            Proto::Http2 => vec!["h2"],
            Proto::Auto => vec!["http/1.1", "h2"]
        };

        let tls_connector = TlsConnector::new_with_tls_default(connector, Some(alpn));
        let mut http_connector = HttpConnector::new(tls_connector);

        if config.proto == Proto::Http1 {
            http_connector.set_http1_only();
        }

        if config.proto == Proto::Http2 {
            http_connector.set_http2_only();
        }

        if let Some(val) = config.initial_max_streams {
            http_connector.h2_builder().initial_max_send_streams(val);
        }

        if let Some(val) = config.max_concurrent_streams {
            http_connector.h2_builder().max_concurrent_streams(val);
        }

        http_connector.set_read_timeout(config.read_timeout);

        let inner = Rc::new(ClientInner {
            config: self.build_config.clone(),
            connector,
            http_connector
        });

        MonoioClient { inner }
    }
}


impl Clone for MonoioClient {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl MonoioClient {
    pub fn make_request(&self) -> HttpRequest {
        HttpRequest::new(self.clone())
    }

    pub(crate) async fn send_request(&self, req: Request<HttpBody>, uri: Uri) -> Result<Response<HttpBody>> {
        let key = uri.try_into().map_err(|e| Error::UriKeyError(e))?;
        let mut conn = self
            .inner
            .http_connector
            .connect(key)
            .await
            .map_err(|e| Error::HttpTransportError(e))?;

        let (res, _) = conn.send_request(req).await;
        res.map_err(|e| Error::HttpResponseError(e))
    }
}