use std::rc::Rc;
use std::time::Duration;

use http::{HeaderMap, HeaderValue, Request, Uri};
use http::header::{CONNECTION, UPGRADE};
use hyper::body::Incoming;
use hyper::client::conn::{http1::Builder as H1Builder, http2::Builder as H2Builder};
use monoio_transports::connectors::{Connector, TcpConnector};
use monoio_transports::connectors::pollio::PollIo;
use monoio_transports::http::hyper::{HyperH1Connection, HyperH1Connector, HyperH2Connection, HyperH2Connector, MonoioExecutor};
use monoio_transports::pool::ConnectionPool;
#[cfg(feature = "hyper-tls")]
use monoio_transports::http::hyper::HyperTlsConnector;

use crate::{
    hyper::hyper_body::HyperBody,
    error::{Error, TransportError},
    request::HttpRequest,
    key::PoolKey,
    Protocol,
    build_connection_pool,
    get_connection_from_connector
};

enum HyperH1ConnectorType {
    #[allow(dead_code)]
    HTTP(HyperH1Connector<PollIo<TcpConnector>, PoolKey, HyperBody>),
    #[cfg(feature = "hyper-tls")]
    HTTPS(HyperH1Connector<HyperTlsConnector<TcpConnector>, PoolKey, HyperBody>)
}

enum HyperH2ConnectorType {
    #[allow(dead_code)]
    HTTP(HyperH2Connector<PollIo<TcpConnector>, PoolKey, HyperBody>),
    #[cfg(feature = "hyper-tls")]
    HTTPS(HyperH2Connector<HyperTlsConnector<TcpConnector>, PoolKey, HyperBody>)
}

#[derive(Default, Clone, Debug)]
struct HyperClientConfig {
    default_headers: Rc<HeaderMap>,
}

impl HyperClientConfig {
    pub fn new(header_map: HeaderMap) -> Self { HyperClientConfig { default_headers: Rc::new(header_map) } }
}

struct HyperClientInner {
    config: HyperClientConfig,
    protocol: Protocol,
    h1_connector: Option<HyperH1ConnectorType>,
    h2_connector: Option<HyperH2ConnectorType>,
}

pub struct MonoioHyperClient {
    inner: Rc<HyperClientInner>,
}

impl MonoioHyperClient {
    pub fn builder() -> HyperClientBuilder {
        HyperClientBuilder::default()
    }
}

impl Clone for MonoioHyperClient {
    fn clone(&self) -> Self {
        MonoioHyperClient {
            inner: self.inner.clone(),
        }
    }
}

#[derive(Default, Clone)]
struct HyperClientBuilderConfig {
    protocol: Protocol,
    pool_disabled: bool,
    enable_https: bool,
    default_headers: HeaderMap,
    max_idle_connections: Option<usize>,
    idle_timeout_duration: Option<Duration>,
    h1_builder: Option<H1Builder>,
    h2_builder: Option<H2Builder<MonoioExecutor>>,
}

#[derive(Default)]
pub struct HyperClientBuilder {
    build_config: HyperClientBuilderConfig,
}

impl HyperClientBuilder {
    /// Disables the connection pooling feature.
    /// When disabled, a new connection will be created for each request.
    pub fn disable_connection_pool(mut self) -> Self {
        self.build_config.pool_disabled = true;
        self
    }

    /// Sets default headers that will be applied to all requests made through this http.
    /// These headers can be overridden by request-specific headers.
    pub fn default_headers(mut self, val: HeaderMap) -> Self {
        self.build_config.default_headers = val;
        self
    }

    /// Sets the maximum number of idle connections that can be kept in the connection pool.
    /// Once this limit is reached, older idle connections will be dropped.
    pub fn max_idle_connections(mut self, val: usize) -> Self {
        self.build_config.max_idle_connections = Some(val);
        self
    }

    /// Sets the duration after which an idle connection in the pool will be closed.
    /// The timeout is specified in seconds.
    pub fn idle_connection_timeout(mut self, val: u64) -> Self {
        self.build_config.idle_timeout_duration = Some(Duration::from_secs(val));
        self
    }

    /// Forces the http to use HTTP/1.1 protocol only, disabling HTTP/2 support.
    /// Useful when you need to ensure HTTP/1.1 compatibility.
    /// Default protocol is Auto
    pub fn http1_only(mut self) -> Self {
        self.build_config.protocol = Protocol::Http1;
        self
    }

    /// Enables HTTP/2 prior knowledge mode, assuming all connections will use HTTP/2.
    /// This skips the HTTP/1.1 -> HTTP/2 upgrade process.
    /// Default protocol is Auto
    pub fn http2_prior_knowledge(mut self) -> Self {
        self.build_config.protocol = Protocol::Http2;
        self
    }

    /// Enables HTTPS/TLS support for secure connections.
    /// Must be called to make HTTPS requests.
    /// Available only on crate features hyper-tls or hyper-native-tls
    #[cfg(feature = "hyper-tls")]
    pub fn enable_https(mut self) -> Self {
        self.build_config.enable_https = true;
        self
    }

    /// Replaces the default HTTP/1.1 builder with a custom configured one.
    /// Useful when you need fine-grained control over HTTP/1.1 connection settings.
    pub fn with_h1_builder(mut self, builder: H1Builder) -> Self {
        self.build_config.h1_builder = Some(builder);
        self
    }

    /// Replaces the default HTTP/2 builder with a custom configured one.
    /// Allows detailed control over HTTP/2-specific settings using the Monoio executor.
    pub fn with_h2_builder(mut self, builder: H2Builder<MonoioExecutor>) -> Self {
        self.build_config.h2_builder = Some(builder);
        self
    }
}

impl HyperClientBuilder {
    pub fn build(&self) -> MonoioHyperClient {
        let build_config = self.build_config.clone();
        let tcp_connector = TcpConnector::default();
        let protocol_h1 = build_config.protocol.is_protocol_h1();
        let protocol_h2 = build_config.protocol.is_protocol_h2();
        let protocol_auto = build_config.protocol.is_protocol_auto();

        let config = if protocol_auto {
            // If protocol is Auto, add connection upgrade headers to default headers for every request
            let mut default_headers = build_config.default_headers.clone();
            default_headers.insert(UPGRADE, HeaderValue::from_static("h2c"));
            default_headers.insert(
                CONNECTION,
                HeaderValue::from_static("Upgrade, HTTP2-Settings"),
            );
            default_headers.insert("HTTP2-Settings", HeaderValue::from_static("AAMAAABkAAQAAP__"));

            HyperClientConfig::new(default_headers)
        } else {
            HyperClientConfig::default()
        };

        let protocol = build_config.protocol.clone();

        // Build H1 connector with connection pool
        let h1_connector = if protocol_h1 || protocol_auto {
            let connection_pool = build_connection_pool!(&build_config);
            let alpn = build_config.enable_https.then(|| vec!["http/1.1"]);

            Self::build_h1_connector(tcp_connector, alpn, connection_pool, build_config.clone())
        } else {
            None
        };

        // Build H2 connector with connection pool
        let h2_connector = if protocol_h2 || protocol_auto {
            let connection_pool = build_connection_pool!(&build_config);
            let alpn = build_config.enable_https.then(|| vec!["h2"]);

            Self::build_h2_connector(tcp_connector, alpn, connection_pool, build_config)
        } else {
            None
        };

        let inner = Rc::new(HyperClientInner {
            config,
            protocol,
            h1_connector,
            h2_connector,
        });

        MonoioHyperClient { inner }
    }

    fn build_h1_connector(
        connector: TcpConnector,
        alpn: Option<Vec<&str>>,
        pool: ConnectionPool<PoolKey, HyperH1Connection<HyperBody>>,
        build_config: HyperClientBuilderConfig
    ) -> Option<HyperH1ConnectorType> {
        match alpn {
            Some(_) => {
                #[cfg(feature = "hyper-tls")]
                {
                    let connector = HyperTlsConnector::new_with_tls_default(connector, alpn);
                    let mut h1 = HyperH1Connector::new_with_pool(connector, pool);
                    if let Some(builder) = build_config.h1_builder {
                        h1 = h1.with_hyper_builder(builder);
                    }
                    return Some(HyperH1ConnectorType::HTTPS(h1));
                }
                #[cfg(not(feature = "hyper-tls"))]
                None
            },
            None => {
                let mut h1 = HyperH1Connector::new_with_pool(PollIo(connector), pool);
                if let Some(builder) = build_config.h1_builder {
                    h1 = h1.with_hyper_builder(builder);
                }
                Some(HyperH1ConnectorType::HTTP(h1))
            }
        }
    }

    fn build_h2_connector(
        connector: TcpConnector,
        alpn: Option<Vec<&str>>,
        pool: ConnectionPool<PoolKey, HyperH2Connection<HyperBody>>,
        build_config: HyperClientBuilderConfig
    ) -> Option<HyperH2ConnectorType> {
        match alpn {
            Some(_) => {
                #[cfg(feature = "hyper-tls")]
                {
                    let connector = HyperTlsConnector::new_with_tls_default(connector, alpn);
                    let mut h2 = HyperH2Connector::new_with_pool(connector, pool);
                    if let Some(builder) = build_config.h2_builder {
                        h2 = h2.with_hyper_builder(builder);
                    }
                    return Some(HyperH2ConnectorType::HTTPS(h2));
                }
                #[cfg(not(feature = "hyper-tls"))]
                None
            },
            None => {
                let mut h2 = HyperH2Connector::new_with_pool(PollIo(connector), pool);
                if let Some(builder) = build_config.h2_builder {
                    h2 = h2.with_hyper_builder(builder);
                }
                Some(HyperH2ConnectorType::HTTP(h2))
            }
        }
    }
}

impl MonoioHyperClient {
    pub fn new_request(&self) -> HttpRequest<MonoioHyperClient> {
        let mut request = HttpRequest::new(self.clone());
        for (key, val) in self.inner.config.default_headers.iter() {
            request = request.set_header(key, val)
        }

        request
    }

    pub(crate) async fn send_request(
        &self,
        req: Request<HyperBody>,
        uri: Uri,
    ) -> Result<http::Response<Incoming>, Error> {
        let key = uri.try_into().map_err(|e| Error::UriKeyError(e))?;

        let response = match self.inner.protocol
        {
            Protocol::Http1 => {
                let mut conn = get_connection_from_connector!(
                    self.inner.h1_connector.as_ref().unwrap(),
                    key
                )?;

                conn.send_request(req).await
            }
            Protocol::Http2 => {
                let mut conn = get_connection_from_connector!(
                    h2 self.inner.h2_connector.as_ref().unwrap(),
                    key
                )?;

                conn.send_request(req).await
            }
            Protocol::Auto => {
                // First create Http/1.1 connection with upgrade headers set
                let mut conn = get_connection_from_connector!(
                    self.inner.h1_connector.as_ref().unwrap(),
                    key.clone()
                )?;

                let maybe_response = conn
                    .send_request(req.clone())
                    .await
                    .map_err(|e| Error::HyperResponseError(e))?;

                // Check if server response contains the upgrade header
                let should_upgrade_to_h2 = maybe_response
                    .headers()
                    .get("upgrade")
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_lowercase().contains("h2c"))
                    .unwrap_or(false);

                if should_upgrade_to_h2 {
                    // Switching to H2 connection
                    let mut conn = get_connection_from_connector!(
                        h2 self.inner.h2_connector.as_ref().unwrap(),
                        key
                    )?;

                    conn.send_request(req).await
                } else {
                    // Return the original H1 response
                    Ok(maybe_response)
                }
            }
        };

        response.map_err(|e| Error::HyperResponseError(e))
    }
}