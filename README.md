# monoio-netreq

`monoio-netreq` is a user-friendly HTTP client library designed for use with the Monoio runtime.  
It is built on top of [monoio-transports](https://github.com/monoio-rs/monoio-transports/tree/master) and [monoio-http](https://github.com/monoio-rs/monoio-http/tree/master/monoio-http).


## Features

- Support for HTTP/1.1 and HTTP/2 protocols.
- TLS support with both `native-tls` and `rustls` for secure connections.
- Connection pooling for efficient resource management.
- Optional feature for a Hyper-based client.
- Hyper client includes TLS support with both `native-tls` and `rustls`.


## Feature Flags

This crate uses `monoio-transports` as a dependency, sourced from this [forked repository](https://github.com/rEflxzR/monoio-transports). By default, the crate re-exports `monoio-transports` from crates.io. Below are the main feature flags available in this crate:

- **default-crate**: Enabled by default. Imports features from the `monoio-transports` crate available on crates.io.
- **pool**: Uses features from the forked Git repository of `monoio-transports`. Enable this flag if you want to customize pool options with the default `HttpConnector`.
- **hyper-tls**: Disables the `io_uring` features of Monoio and enables the `tokio-compat` legacy feature. Use this only if you need TLS support with `HyperConnectors`.

### Additional Features

Other available feature flags include:
- `hyper`
- `native-tls`
- `pool-hyper`
- `pool-native-tls`
- `hyper-native-tls`

All Hyper-related features are gated behind the `hyper` flag. The `native-tls` feature enables native TLS support, while `rustls` is used as the default TLS implementation.


## Examples

For implementation details, please refer to [examples](./examples)

(Credits to respective authors for the monoio packages)