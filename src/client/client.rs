use std::rc::Rc;
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

struct ClientInner {
    config: Config,
    connector: TcpConnector,
    http_connector: HttpConnector<TlsConnector<TcpConnector>, TcpTlsAddr, TlsStream<TcpStream>>,
}

#[derive(Default, Clone)]
struct Config {
    proto: Proto,
    pool_disabled: bool,
    max_idle_connections: usize,
    idle_timeout_duration: u64,
}


impl MonoioClient {
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }
}

#[derive(Default)]
struct ClientBuilder {
    build_config: Config
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

    pub fn idle_connection_timeout_duration(mut self, val: u64) -> Self {
        self.build_config.idle_timeout_duration = val;
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
        let http_connector = HttpConnector::new(tls_connector);

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

        conn.send_request(req).await.0.map_err(|e| Error::HttpResponseError(e))
    }
}