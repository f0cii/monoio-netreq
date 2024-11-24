use super::error::Error;
use bytes::Bytes;
use http::{Extensions, HeaderMap, HeaderValue, StatusCode, Version};
#[cfg(feature = "hyper")]
use http_body_util::BodyExt as HyperBodyExt;
#[cfg(feature = "hyper")]
use hyper::body::Incoming;
use monoio_http::common::body::{BodyExt, HttpBody};
use monoio_http::h1::payload::Payload;

pub type Response<P = Payload> = http::response::Response<P>;

#[derive(Debug)]
pub struct HttpResponse<B> {
    status: StatusCode,
    version: Version,
    headers: HeaderMap<HeaderValue>,
    extensions: Extensions,
    body: B,
}

impl<B> HttpResponse<B> {
    pub fn status(&self) -> StatusCode {
        self.status
    }

    pub fn version(&self) -> Version {
        self.version
    }

    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    pub fn extensions(&self) -> &Extensions {
        &self.extensions
    }

    pub fn raw_body(self) -> B {
        self.body
    }
}

impl HttpResponse<HttpBody> {
    pub(crate) fn new(response: Response<HttpBody>) -> Self {
        let (parts, body) = response.into_parts();

        HttpResponse {
            status: parts.status,
            version: parts.version,
            headers: parts.headers,
            extensions: parts.extensions,
            body,
        }
    }

    pub async fn bytes(self) -> Result<Bytes, Error> {
        let body = self.body;
        body
            .bytes()
            .await
            .map_err(|e| Error::BytesError(e.to_string()))
    }

    pub async fn json<T: serde::de::DeserializeOwned>(self) -> Result<T, Error> {
        let bytes = self
            .body
            .bytes()
            .await
            .map_err(|e| Error::BytesError(e.to_string()))?;
        let d = serde_json::from_slice(&bytes).map_err(|e| Error::SerdeDeserializeError(e))?;

        Ok(d)
    }
}

#[cfg(feature = "hyper")]
impl HttpResponse<Bytes> {
    pub(crate) async fn hyper_new(response: Response<Incoming>) -> Result<Self, Error> {
        let (parts, byte_stream) = response.into_parts();
        let body = byte_stream
            .collect()
            .await
            .map_err(|e| Error::BytesError(e.to_string()))?
            .to_bytes();

        Ok(HttpResponse {
            status: parts.status,
            version: parts.version,
            headers: parts.headers,
            extensions: parts.extensions,
            body,
        })
    }

    pub async fn json<T: serde::de::DeserializeOwned>(self) -> Result<T, Error> {
        let d = serde_json::from_slice(&self.body).map_err(|e| Error::SerdeDeserializeError(e))?;

        Ok(d)
    }
}
