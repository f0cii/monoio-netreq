use std::any::Any;
use bytes::Bytes;
use http::{HeaderName, HeaderValue, Method, Request, Uri, Version};
use http::request::Builder;
use http::header::{CONNECTION, HOST, TE, TRANSFER_ENCODING, UPGRADE};
use monoio_http::common::body::{FixedBody, HttpBody};

use super::client::http::MonoioClient;
#[cfg(feature = "hyper")]
use super::client::hyper::MonoioHyperClient;
#[cfg(feature = "hyper")]
use super::client::hyper_body::HyperBody;
use super::error::Error;
use super::response::HttpResponse;
#[cfg(feature = "hyper")]
use super::response::HyperResponse;

const PROHIBITED_HEADERS: [HeaderName; 5] = [
    CONNECTION,
    HeaderName::from_static("keep-alive"),
    TE,
    TRANSFER_ENCODING,
    UPGRADE
];

pub struct HttpRequest<C> {
    client: C,
    builder: Builder
}

impl<C> HttpRequest<C> {
    pub(crate) fn new(client: C) -> HttpRequest<C> {
        HttpRequest { client, builder: Builder::default() }
    }

    pub fn set_uri<T>(mut self, uri: T) -> Self
        where
            Uri: TryFrom<T>,
            <Uri as TryFrom<T>>::Error: Into<http::Error>,
    {
        self.builder = self.builder.uri(uri);
        self
    }

    pub fn set_method<T>(mut self, method: T) -> Self
        where
            Method: TryFrom<T>,
            <Method as TryFrom<T>>::Error: Into<http::Error>,
    {
        self.builder = self.builder.method(method);
        self
    }

    /// Connection specific headers will be removed from http2 requests.
    /// http2 also restricts 'host' as header. host is mandatory in http1 and added by default if not set
    pub fn set_header<K, T>(mut self, key: K, value: T) -> Self
        where
            HeaderName: TryFrom<K>,
            <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
            HeaderValue: TryFrom<T>,
            <HeaderValue as TryFrom<T>>::Error: Into<http::Error>,
    {
        self.builder = self.builder.header(key, value);
        self
    }

    /// Sets the http request version. Default is HTTP_11
    pub fn set_version(mut self, version: Version) -> Self {
        self.builder = self.builder.version(version);
        self
    }

    pub fn set_extension<T>(mut self, extension: T) -> Self
        where
            T: Clone + Any + Send + Sync + 'static,
    {
        self.builder = self.builder.extension(extension);
        self
    }
}

impl HttpRequest<MonoioClient> {
    fn build_http_request(builder: Builder, body: Option<Bytes>) -> Result<(Request<HttpBody>, Uri), Error> {
        let mut http_request = builder
            .body(HttpBody::fixed_body(body))
            .map_err(|e| Error::HttpRequestBuilder(e))?;

        let uri = http_request.uri().clone();

        // Remove any connection specific headers to Http/2 requests
        // Avoid adding host header to Http/2 based requests but not Http/1.1
        // unless you are sending request to a proxy which downgrade the connection
        match http_request.version() {
            Version::HTTP_2 | Version::HTTP_3 => {
                let headers = http_request.headers_mut();
                for header in PROHIBITED_HEADERS.iter() {
                    headers.remove(header);
                }
            },
            _ => {
                if let Some(host) = uri.host() {
                    let host = HeaderValue::try_from(host).map_err(|e| Error::InvalidHeaderValue(e))?;
                    let headers = http_request.headers_mut();
                    if !headers.contains_key(HOST) {
                        headers.insert(HOST, host);
                    }
                }
            }
        }

        Ok((http_request, uri))
    }

    /// Builds and sends a request without any request body
    pub async fn send(self) -> Result<HttpResponse, Error> {
        let (req, uri) = HttpRequest::build_http_request(self.builder, None)?;
        let http_response = self
            .client
            .send_request(req, uri)
            .await?;
        Ok(HttpResponse::new(http_response))
    }

    /// Builds and sends a request with provided body. Must be converted to bytes
    pub async fn send_body(self, body: Bytes) -> Result<HttpResponse, Error> {
        let (req, uri) = HttpRequest::build_http_request(self.builder, Some(body))?;
        let http_response = self
            .client
            .send_request(req, uri)
            .await?;
        Ok(HttpResponse::new(http_response))
    }
}

#[cfg(feature = "hyper")]
impl HttpRequest<MonoioHyperClient> {
    fn build_hyper_request(builder: Builder, body: Bytes) -> Result<(Request<HyperBody>, Uri), Error> {
        let hyper_body = HyperBody::from(body);
        let mut hyper_request = builder
            .body(hyper_body)
            .map_err(|e| Error::HttpRequestBuilder(e))?;

        let uri = hyper_request.uri().clone();

        match hyper_request.version() {
            Version::HTTP_2 | Version::HTTP_3 => {
                let headers = hyper_request.headers_mut();
                for header in PROHIBITED_HEADERS.iter() {
                    headers.remove(header);
                }
            },
            _ => {
                if let Some(host) = uri.host() {
                    let host = HeaderValue::try_from(host).map_err(|e| Error::InvalidHeaderValue(e))?;
                    let headers = hyper_request.headers_mut();
                    if !headers.contains_key(HOST) {
                        headers.insert(HOST, host);
                    }
                }
            }
        }

        Ok((hyper_request, uri))
    }

    /// Build and sends a request
    pub async fn send(self, body: Bytes) -> Result<HyperResponse, Error> {
        let (req, uri) = HttpRequest::build_hyper_request(self.builder, body)?;
        let http_response = self
            .client
            .send_request(req, uri)
            .await?;
        Ok(HyperResponse::new(http_response))
    }
}