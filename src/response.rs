use bytes::Bytes;
use http::{Extensions, HeaderMap, HeaderValue, StatusCode, Version};
use monoio_http::common::body::{BodyExt, HttpBody};
use monoio_http::h1::payload::Payload;
use super::error::Error;
#[cfg(feature = "hyper")]
use hyper::body::Incoming;
#[cfg(feature = "hyper")]
use http::response::Parts;

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
    pub(crate) fn new(response: Response<HttpBody>) -> Self {
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

#[cfg(feature = "hyper")]
#[derive(Debug)]
pub struct HyperResponse {
    parts: Parts,
    _body: Incoming,
}

#[cfg(feature = "hyper")]
impl HyperResponse {
    pub(crate) fn new(response: http::Response<Incoming>) -> Self {
        let (parts, _body) = response.into_parts();
        HyperResponse { parts, _body }
    }

    pub fn status(&self) -> StatusCode { self.parts.status }

    pub fn version(&self) -> Version { self.parts.version }

    pub fn headers(&self) -> &HeaderMap { &self.parts.headers }

    pub fn extensions(&self) -> &Extensions { &self.parts.extensions }

    // TODO: Fix the Incoming body not an iterator error
    // pub async fn json<T: serde::de::DeserializeOwned>(self) -> Result<T, Error> {
    //     let body = self.body.collect().await.map_err(|e| Error::ByteDecodeError(e))?.aggregate();
    //     let d = serde_json::from_reader(body.reader()).map_err(|e| Error::SerdeDeserializeError(e))?;
    //
    //     Ok(d)
    // }
}