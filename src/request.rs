use bytes::Bytes;
use http::header::{CONNECTION, HOST, TE, TRANSFER_ENCODING, UPGRADE};
use http::request::Builder;
use http::{HeaderName, HeaderValue, Method, Request, Uri, Version};
use std::any::Any;
use monoio_http::common::body::HttpBody;

use super::client::http::MonoioClient;
#[cfg(feature = "hyper")]
use super::client::hyper::MonoioHyperClient;
#[cfg(feature = "hyper")]
use super::client::hyper_body::HyperBody;
use super::client::monoio_body::MonoioBody;
use super::error::Error;
use super::response::HttpResponse;

const PROHIBITED_HEADERS: [HeaderName; 5] = [
    CONNECTION,
    HeaderName::from_static("keep-alive"),
    TE,
    TRANSFER_ENCODING,
    UPGRADE,
];

pub trait RequestBody {
    type Body;

    fn create_body(bytes: Option<Bytes>) -> Self::Body;
}

pub struct HttpRequest<C> {
    client: C,
    builder: Builder,
}

impl<C> HttpRequest<C> {
    pub(crate) fn new(client: C) -> HttpRequest<C> {
        HttpRequest {
            client,
            builder: Builder::default(),
        }
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

    fn build_request<B: RequestBody>(
        builder: Builder,
        body: Option<Bytes>,
    ) -> Result<(Request<B::Body>, Uri), Error> {
        let mut request = builder
            .body(B::create_body(body))
            .map_err(Error::HttpRequestBuilder)?;

        let uri = request.uri().clone();

        // Remove any connection specific headers to Http/2 requests
        // Avoid adding host header to Http/2 based requests but not Http/1.1
        // unless you are sending request to a proxy which downgrade the connection
        match request.version() {
            Version::HTTP_2 | Version::HTTP_3 => {
                let headers = request.headers_mut();
                for header in PROHIBITED_HEADERS.iter() {
                    headers.remove(header);
                }
            }
            _ => {
                if let Some(host) = uri.host() {
                    let host = HeaderValue::try_from(host).map_err(Error::InvalidHeaderValue)?;
                    let headers = request.headers_mut();
                    if !headers.contains_key(HOST) {
                        headers.insert(HOST, host);
                    }
                }
            }
        }

        Ok((request, uri))
    }
}

impl HttpRequest<MonoioClient> {
    pub async fn send(self) -> Result<HttpResponse<HttpBody>, Error> {
        self.send_body(None).await
    }

    pub async fn send_body(self, body: impl Into<Option<Bytes>>) -> Result<HttpResponse<HttpBody>, Error> {
        let (req, uri) = Self::build_request::<MonoioBody>(self.builder, body.into())?;
        let response = self.client.send_request(req, uri).await?;
        Ok(HttpResponse::new(response))
    }
}

#[cfg(feature = "hyper")]
impl HttpRequest<MonoioHyperClient> {
    pub async fn send(self) -> Result<HttpResponse<Bytes>, Error> { self.send_body(None).await }

    pub async fn send_body(self, body: impl Into<Option<Bytes>>) -> Result<HttpResponse<Bytes>, Error> {
        let (req, uri) = Self::build_request::<HyperBody>(self.builder, body.into())?;
        let response = self.client.send_request(req, uri).await?;
        HttpResponse::hyper_new(response).await
    }
}
