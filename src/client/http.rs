use std::rc::Rc;
use std::time::Duration;
use http::{HeaderMap, Request, Uri};
use monoio::net::TcpStream;
use monoio_http::common::body::HttpBody;
use monoio_transports::http::{HttpConnector};
use monoio_transports::connectors::{Connector, TcpConnector, TlsStream, TcpTlsAddr as TlsKey};
use monoio_transports::connectors::TlsConnector;

use super::Protocol;
use super::key::TcpAddr as Key;
use crate::response::{Response};
use crate::error::{Error, Result};
use crate::request::HttpRequest;

enum HttpConnectorType {
    HTTP(HttpConnector<TcpConnector, Key, TcpStream>),
    HTTPS(HttpConnector<TlsConnector<TcpConnector>, TlsKey, TlsStream<TcpStream>>)
}

#[derive(Default, Clone, Debug)]
struct ClientConfig {
    default_headers: Rc<HeaderMap>,
}

struct ClientInner {
    config: ClientConfig,
    http_connector: HttpConnectorType,
}

pub struct MonoioClient {
    inner: Rc<ClientInner>,
}

impl MonoioClient {
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }
}

impl Clone for MonoioClient {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

// TODO: Can we include monoio-transports locally and allow customized client pool sizes ?
#[derive(Default, Clone)]
struct ClientBuilderConfig {
    protocol: Protocol,
    enable_https: bool,
    pool_disabled: bool,
    max_idle_connections: usize,
    idle_timeout_duration: u32,
    read_timeout: Option<Duration>,
    initial_max_streams: Option<usize>,
    max_concurrent_streams: Option<u32>,
    default_headers: HeaderMap,
}

#[derive(Default)]
pub struct ClientBuilder {
    build_config: ClientBuilderConfig
}

impl ClientBuilder {
    /// Sets the default headers. These headers will be set for every request by the client.
    pub fn default_headers(mut self, val: HeaderMap) -> Self {
        self.build_config.default_headers = val;
        self
    }
    /// Disables connection pooling for the client
    pub fn disable_connection_pool(mut self) -> Self {
        self.build_config.pool_disabled = true;
        self
    }

    /// Sets the number of idle connections allowed in the pool
    pub fn max_idle_connections(mut self, val: usize) -> Self {
        self.build_config.max_idle_connections = val;
        self
    }

    /// Sets timeout in seconds for idle connections before they are removed from the pool
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

    /// Set the max number of concurrent streams per connection. Default is 100
    pub fn max_concurrent_streams(mut self, val: u32) -> Self {
        self.build_config.max_concurrent_streams = Some(val);
        self
    }

    /// Sets the client protocol to use HTTP1 only. Default is Auto
    pub fn http1_only(mut self) -> Self {
        self.build_config.protocol = Protocol::Http1;
        self
    }

    /// Sets the client protocol to use HTTP2 only. Default is Auto
    pub fn http2_prior_knowledge(mut self) -> Self {
        self.build_config.protocol = Protocol::Http2;
        self
    }

    /// Enables support for https scheme. Default is http only
    pub fn enable_https(mut self) -> Self {
        self.build_config.enable_https = true;
        self
    }
}

macro_rules! apply_parameter_from_config {
    ($connector:expr, $method:ident($val:expr)) => {
        match $connector {
            HttpConnectorType::HTTP(ref mut c) => c.$method($val),
            HttpConnectorType::HTTPS(ref mut c) => c.$method($val),
        }
    };

    ($connector:expr, $builder:ident().$method:ident($val:expr)) => {
        match $connector {
            HttpConnectorType::HTTP(ref mut c) => c.$builder().$method($val),
            HttpConnectorType::HTTPS(ref mut c) => c.$builder().$method($val),
        }
    };
}

impl ClientBuilder {
    pub fn build(self) -> MonoioClient {
        let build_config = self.build_config.clone();
        let config = ClientConfig::default();
        let tcp_connector = TcpConnector::default();

        let mut http_connector = if build_config.enable_https {
            // TLS based Connector. Client will negotiate the connection using ALPN, no need to set Protocols.
            let alpn = match build_config.protocol {
                Protocol::Http1 => vec!["http/1.1"],
                Protocol::Http2 => vec!["h2"],
                Protocol::Auto => vec!["http/1.1", "h2"]
            };

            let tls_connector = TlsConnector::new_with_tls_default(tcp_connector, Some(alpn));

            HttpConnectorType::HTTPS(HttpConnector::new(tls_connector))
        } else {
            // Default TCP based Connector
            let mut connector = HttpConnector::new(tcp_connector);

            if build_config.protocol.is_protocol_h1() {
                connector.set_http1_only();
            }

            // Assumes http2 prior knowledge
            if build_config.protocol.is_protocol_h2() {
                connector.set_http2_only();
            }

            HttpConnectorType::HTTP(connector)
        };

        if let Some(val) = build_config.initial_max_streams {
            apply_parameter_from_config!(http_connector, h2_builder().initial_max_send_streams(val));
        }

        if let Some(val) = build_config.max_concurrent_streams {
            apply_parameter_from_config!(http_connector, h2_builder().max_concurrent_streams(val));
        }

        apply_parameter_from_config!(http_connector, set_read_timeout(build_config.read_timeout));

        let inner = Rc::new(ClientInner {
            config,
            http_connector
        });

        MonoioClient { inner }
    }
}

impl MonoioClient {
    /// Returns a new http request with default parameters
    pub fn make_request(&self) -> HttpRequest<MonoioClient> {
        let mut request = HttpRequest::new(self.clone());
        for (key, val) in self.inner.config.default_headers.iter() {
            request = request.set_header(key, val)
        }

        request
    }

    pub(crate) async fn send_request(
        &self,
        req: Request<HttpBody>,
        uri: Uri
    ) -> Result<Response<HttpBody>> {
        // The connection pool keys for Non TLS and TLS based connectors are slightly different
        let (response, _) = match self.inner.http_connector {
            HttpConnectorType::HTTP(ref connector) => {
                let key = uri
                    .try_into()
                    .map_err(|e| Error::UriKeyError(e))?;
                let mut conn = connector
                    .connect(key)
                    .await
                    .map_err(|e| Error::HttpTransportError(e))?;
                conn.send_request(req).await
            },

            HttpConnectorType::HTTPS(ref connector) => {
                let key = uri
                    .try_into()
                    .map_err(|e| Error::UriKeyError(e))?;
                let mut conn = connector
                    .connect(key)
                    .await
                    .map_err(|e| Error::HttpTransportError(e))?;
                conn.send_request(req).await
            }
        };

        response.map_err(|e| Error::HttpResponseError(e))
    }
}