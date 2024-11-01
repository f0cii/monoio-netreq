use monoio_http::common::error::HttpError;
use thiserror::Error as ThisError;

pub type Result<T> = std::result::Result<T, HttpError>;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("error building http request: {0:?}")]
    HttpRequestBuilder(http::Error),
    #[error("{0:?}")]
    HttpResponseError(HttpError),
    #[error("{0:?}")]
    ByteDecodeError(HttpError),
    #[error("serde body deserialize error: {0:?}")]
    SerdeDeserializeError(serde_json::Error),
}

// impl std::fmt::Display for Error {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Error::HttpRequestBuilder(e) => write!(f, "error building http request: {}", e),
//             Error::HttpResponseError(e) => write!(f, "error processing http response: {}", e),
//             Error::ByteDecodeError(e) => write!(f, "error converting response body to bytes: {}", e),
//             Error::SerdeDeserializeError(e) => write!(f, "error during serde deserializing: {}", e),
//         }
//     }
// }
