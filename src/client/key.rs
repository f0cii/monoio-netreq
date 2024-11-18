// Borrowed from TcpTlsAddrs for Non Tls Pool Keys
use http::Uri;
use monoio_transports::FromUriError;
use std::net::ToSocketAddrs;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TcpAddr {
    pub host: smol_str::SmolStr,
    pub port: u16,
}

impl ToSocketAddrs for TcpAddr {
    type Iter = <(&'static str, u16) as ToSocketAddrs>::Iter;

    #[inline]
    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        (self.host.as_str(), self.port).to_socket_addrs()
    }
}

impl TryFrom<&Uri> for TcpAddr {
    type Error = FromUriError;

    #[inline]
    fn try_from(uri: &Uri) -> Result<Self, Self::Error> {
        let host = match uri.host() {
            Some(a) => a,
            None => return Err(FromUriError::NoAuthority),
        };

        let (tls, default_port) = match uri.scheme() {
            Some(scheme) if scheme == &http::uri::Scheme::HTTP => (false, 80),
            Some(scheme) if scheme == &http::uri::Scheme::HTTPS => (true, 443),
            _ => (false, 0),
        };
        if tls {
            return Err(FromUriError::UnsupportScheme);
        }
        let host = smol_str::SmolStr::from(host);
        let port = uri.port_u16().unwrap_or(default_port);

        Ok(TcpAddr { host, port })
    }
}

impl TryFrom<Uri> for TcpAddr {
    type Error = FromUriError;

    #[inline]
    fn try_from(value: Uri) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}
