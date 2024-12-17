use http::Error as HttpError;
use http::header::InvalidHeaderValue;
use monoio_transports::{FromUriError, TransportError as MonoioTransportError};
#[cfg(not(feature = "hyper-tls"))]
use monoio_http::common::error::HttpError as MonoioHttpError;
#[cfg(any(feature = "hyper", feature = "pool-hyper", feature = "hyper-tls"))]
use monoio_transports::{
    connectors::pollio::PollConnectError,
    http::hyper::HyperError
};
#[cfg(feature = "hyper-tls")]
use monoio_transports::http::hyper::TlsError;
use serde_json::Error as SerdeError;
use thiserror::{Error as ThisError, Error};

#[cfg(not(feature = "hyper-tls"))]
pub type Result<T> = std::result::Result<T, Error>;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("{0:?}")]
    InvalidHeaderValue(InvalidHeaderValue),
    #[error("error building http request: {0:?}")]
    HttpRequestBuilder(HttpError),
    #[error("http request version and http protocol does not match: {0:?}")]
    HttpVersionMismatch(String),
    #[error("error making pool key from uri: {0:?}")]
    UriKeyError(FromUriError),
    #[error("{0:?}")]
    TransportError(TransportError),
    #[cfg(not(feature = "hyper-tls"))]
    #[error("{0:?}")]
    HttpResponseError(MonoioHttpError),
    #[cfg(any(feature = "hyper", feature = "pool-hyper", feature = "hyper-tls"))]
    #[error("{0:?}")]
    HyperResponseError(hyper::Error),
    #[error("{0:?}")]
    BytesError(String),
    #[error("serde body deserialize error: {0:?}")]
    SerdeDeserializeError(SerdeError),
    #[error("Hyper Connector was not initialized")]
    ConnectorNotInitialized,
}

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("http connector error: {0:?}")]
    HttpConnectorError(MonoioTransportError),
    #[cfg(any(feature = "hyper", feature = "pool-hyper", feature = "hyper-tls"))]
    #[error("hyper poll error: {0:?}")]
    HyperPollError(HyperError<PollConnectError<std::io::Error>>),
    #[cfg(feature = "hyper-tls")]
    #[error("Hyper TLS stream error: {0:?}")]
    TlsStreamError(HyperError<TlsError>),
}

impl From<TransportError> for Error {
    fn from(err: TransportError) -> Self {
        Error::TransportError(err)
    }
}