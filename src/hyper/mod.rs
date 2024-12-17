pub(crate) mod hyper_body;
pub mod client;

#[macro_export]
macro_rules! build_connection_pool {
    (&$build_config:expr) => {
        if $build_config.pool_disabled {
                ConnectionPool::new(Some(0))
            } else {
                ConnectionPool::new_with_idle_interval(
                    $build_config.idle_timeout_duration,
                    $build_config.max_idle_connections
                )
            }
    };
}

#[macro_export]
macro_rules! get_connection_from_connector {
    ($connector:expr, $key:expr) => {{
        match $connector {
            HyperH1ConnectorType::HTTP(connector) => {
                connector
                    .connect($key)
                    .await
                    .map_err(|e| TransportError::HyperPollError(e))
            },
            #[cfg(feature = "hyper-tls")]
            HyperH1ConnectorType::HTTPS(connector) => {
                connector
                    .connect($key)
                    .await
                    .map_err(|e| TransportError::TlsStreamError(e))
            }
        }
    }};

    (h2 $connector:expr, $key:expr) => {{
        match $connector {
            HyperH2ConnectorType::HTTP(connector) => {
                connector
                    .connect($key)
                    .await
                    .map_err(|e| TransportError::HyperPollError(e))
            },
            #[cfg(feature = "hyper-tls")]
            HyperH2ConnectorType::HTTPS(connector) => {
                connector
                    .connect($key)
                    .await
                    .map_err(|e| TransportError::TlsStreamError(e))
            }
        }
    }};
}