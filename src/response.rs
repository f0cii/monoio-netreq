use bytes::Bytes;
use http::{Extensions, HeaderMap, HeaderValue, StatusCode, Version};
use monoio_http::common::body::{BodyExt, HttpBody};
use monoio_http::h1::payload::Payload;
use super::error::Error;

pub type Response<P = Payload> = http::response::Response<P>;

#[derive(Debug)]
pub struct HttpResponse {
    status: StatusCode,
    version: Version,
    headers: HeaderMap<HeaderValue>,
    extensions: Extensions,
    body: HttpBody,
}

impl HttpResponse {
    pub fn new(response: Response<HttpBody>) -> Self {
        let (parts, body) = response.into_parts();

        HttpResponse {
            status: parts.status,
            version: parts.version,
            headers: parts.headers,
            extensions: parts.extensions,
            body
        }
    }

    pub fn status(&self) -> StatusCode { self.status }

    pub fn version(&self) -> Version { self.version }

    pub fn headers(&self) -> &HeaderMap { &self.headers }

    pub fn extensions(&self) -> &Extensions { &self.extensions }

    pub async fn bytes(self) -> Result<Bytes, Error> {
        let body = self.body;
        body
            .bytes()
            .await
            .map_err(|e| Error::ByteDecodeError(e))
    }

    pub fn raw_body(self) -> HttpBody { self.body }

    pub async fn json<T: serde::de::DeserializeOwned>(self) -> Result<T, Error> {
        let bytes = self
            .body
            .bytes()
            .await
            .map_err(|e| Error::ByteDecodeError(e))?;
        let d = serde_json::from_slice(&bytes).map_err(|e| Error::SerdeDeserializeError(e))?;

        Ok(d)
    }
}