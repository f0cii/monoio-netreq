use std::any::Any;
use bytes::Bytes;
use http::{HeaderName, HeaderValue, Method, Uri, Version};
use http::request::Builder;
use monoio_http::common::body::{FixedBody, HttpBody};

use super::client::client::MonoioClient;
use super::error::Error;
use super::response::HttpResponse;

pub struct HttpRequest {
    client: MonoioClient,
    builder: Builder
}

impl HttpRequest {
    pub fn new(client: MonoioClient) -> HttpRequest {
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

    pub fn set_version<T>(mut self, version: Version) -> Self {
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

    pub async fn send(self) -> Result<HttpResponse, Error> {
        let http_request = self
            .builder
            .body(HttpBody::fixed_body(None))
            .map_err(|e| Error::HttpRequestBuilder(e))?;
        let uri = http_request.uri().clone();

        let http_response = self
            .client
            .send_request(http_request, uri)
            .await
            .map_err(|e| {
                Error::HttpResponseError(e)
            })?;
        Ok(HttpResponse::new(http_response))
    }

    pub async fn send_body<T>(self, body: Bytes) -> Result<HttpResponse, Error> {
        let http_request = self
            .builder
            .body(HttpBody::fixed_body(Some(body)))
            .map_err(|e| Error::HttpRequestBuilder(e))?;
        let uri = http_request.uri().clone();

        let http_response = self
            .client
            .send_request(http_request, uri)
            .await
            .map_err(|e| {
                Error::HttpResponseError(e)
            })?;
        Ok(HttpResponse::new(http_response))
    }
}