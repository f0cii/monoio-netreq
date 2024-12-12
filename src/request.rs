use std::any::Any;

use bytes::Bytes;
use http::{HeaderName, HeaderValue, Method, Request, Uri, Version};
use http::header::{CONNECTION, HOST, TE, TRANSFER_ENCODING, UPGRADE};
use http::request::Builder;
use monoio_http::common::body::HttpBody;

#[cfg(any(feature = "hyper", feature = "hyper-patch"))]
use crate::hyper::client::MonoioHyperClient;
#[cfg(any(feature = "hyper", feature = "hyper-patch"))]
use crate::hyper::hyper_body::HyperBody;

use super::{
    http::client::MonoioClient,
    http::monoio_body::MonoioBody,
    response::HttpResponse,
    error::Error,
};

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

    /// Sets the URI for the HTTP request.
    /// Accepts any type that can be converted into a `Uri`.
    /// # Examples
    /// ```
    /// request.set_uri("https://example.com/path");
    /// request.set_uri(Uri::from_static("https://example.com/path"));
    /// ```
    pub fn set_uri<T>(mut self, uri: T) -> Self
        where
            Uri: TryFrom<T>,
            <Uri as TryFrom<T>>::Error: Into<http::Error>,
    {
        self.builder = self.builder.uri(uri);
        self
    }

    /// Sets the HTTP method for the request (GET, POST, PUT, etc.).
    /// Accepts any type that can be converted into a `Method`.
    /// # Examples
    /// ```
    /// request.set_method("POST");
    /// request.set_method(Method::POST);
    /// ```
    pub fn set_method<T>(mut self, method: T) -> Self
        where
            Method: TryFrom<T>,
            <Method as TryFrom<T>>::Error: Into<http::Error>,
    {
        self.builder = self.builder.method(method);
        self
    }

    /// Sets a header in the HTTP request.
    /// Note: For HTTP/2 requests, connection-specific headers will be automatically removed.
    /// The 'host' header is mandatory in HTTP/1.1 and will be added by default if not set.
    /// # Examples
    /// ```
    /// request.set_header("content-type", "application/json");
    /// request.set_header(HeaderName::from_static("authorization"), "Bearer token");
    /// ```
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

    /// Sets the HTTP version for the request.
    /// Default version is HTTP/1.1 if not specified.
    /// # Examples
    /// ```
    /// request.set_version(Version::HTTP_11);
    /// request.set_version(Version::HTTP_2);
    /// ```
    pub fn set_version(mut self, version: Version) -> Self {
        self.builder = self.builder.version(version);
        self
    }

    /// Adds a type-based extension to the request.
    /// Extensions can be used to store extra information that travels along with the request.
    /// The extension type must be `Clone + Any + 'static`.
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
    /// Sends the HTTP request without a body.
    /// Returns a Result containing either the HTTP response or an error.
    pub async fn send(self) -> Result<HttpResponse<HttpBody>, Error> {
        self.send_body(None).await
    }

    /// Sends the HTTP request with an optional body.
    /// The body can be provided as any type that can be converted into `Option<Bytes>`.
    /// Returns a Result containing either the HTTP response or an error.
    /// # Examples
    /// ```
    /// let response = request.send_body(Some(Bytes::from("request body"))).await?;
    /// let response = request.send_body(None).await?; // No body
    /// ```
    pub async fn send_body(self, body: impl Into<Option<Bytes>>) -> Result<HttpResponse<HttpBody>, Error> {
        let (req, uri) = Self::build_request::<MonoioBody>(self.builder, body.into())?;
        let response = self.client.send_request(req, uri).await?;
        Ok(HttpResponse::new(response))
    }
}

#[cfg(any(feature = "hyper", feature = "hyper-patch"))]
impl HttpRequest<MonoioHyperClient> {
    /// Sends the HTTP request without a body.
    /// Returns a Result containing either the HTTP response or an error.
    pub async fn send(self) -> Result<HttpResponse<Bytes>, Error> { self.send_body(None).await }

    /// Sends the HTTP request with an optional body.
    /// The body can be provided as any type that can be converted into `Option<Bytes>`.
    /// Returns a Result containing either the HTTP response or an error.
    /// # Examples
    /// ```
    /// let response = request.send_body(Some(Bytes::from("request body"))).await?;
    /// let response = request.send_body(None).await?; // No body
    /// ```
    pub async fn send_body(self, body: impl Into<Option<Bytes>>) -> Result<HttpResponse<Bytes>, Error> {
        let (req, uri) = Self::build_request::<HyperBody>(self.builder, body.into())?;
        let response = self.client.send_request(req, uri).await?;
        HttpResponse::hyper_new(response).await
    }
}
