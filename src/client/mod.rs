pub mod http_conn;
mod key;

#[derive(Default, Clone, PartialEq, Debug)]
enum Proto {
    Http1,
    Http2,
    #[default]
    Auto
}