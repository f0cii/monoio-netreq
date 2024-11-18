use http::header::InvalidHeaderValue;
use http::Error as HttpError;
use monoio_http::common::error::HttpError as MonoioHttpError;
#[cfg(feature = "hyper")]
use monoio_transports::connectors::pollio::PollConnectError;
#[cfg(feature = "hyper")]
use monoio_transports::http::hyper::HyperError;
use monoio_transports::{FromUriError, TransportError};
use serde_json::Error as SerdeError;
use thiserror::Error as ThisError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("{0:?}")]
    InvalidHeaderValue(InvalidHeaderValue),
    #[error("error building http request: {0:?}")]
    HttpRequestBuilder(HttpError),
    #[error("http request version and client protocol does not match: {0:?}")]
    HttpVersionMismatch(String),
    #[error("error making pool key from uri: {0:?}")]
    UriKeyError(FromUriError),
    #[error("http transport error requesting a connection: {0:?}")]
    HttpTransportError(TransportError),
    #[cfg(feature = "hyper")]
    #[error("hyper transport error requesting a connection: {0:?}")]
    HyperTransportError(HyperError<PollConnectError<std::io::Error>>),
    #[error("{0:?}")]
    HttpResponseError(MonoioHttpError),
    #[cfg(feature = "hyper")]
    #[error("{0:?}")]
    HyperResponseError(hyper::Error),
    #[error("{0:?}")]
    ByteDecodeError(String),
    #[error("serde body deserialize error: {0:?}")]
    SerdeDeserializeError(SerdeError),
}
