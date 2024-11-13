use monoio_http::common::error::HttpError as MonoioHttpError;
use http::Error as HttpError;
use http::header::InvalidHeaderValue;
use serde_json::Error as SerdeError;
use monoio_transports::{FromUriError, TransportError};
use thiserror::Error as ThisError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("{0:?}")]
    InvalidHeaderValue(InvalidHeaderValue),
    #[error("error building http request: {0:?}")]
    HttpRequestBuilder(HttpError),
    #[error("http request version and client alpn does not match: {0:?}")]
    HttpVersionMismatch(String),
    #[error("error making pool key from uri: {0:?}")]
    UriKeyError(FromUriError),
    #[error("transport error requesting a connection: {0:?}")]
    HttpTransportError(TransportError),
    #[error("{0:?}")]
    HttpResponseError(MonoioHttpError),
    #[error("{0:?}")]
    ByteDecodeError(MonoioHttpError),
    #[error("serde body deserialize error: {0:?}")]
    SerdeDeserializeError(SerdeError),
}