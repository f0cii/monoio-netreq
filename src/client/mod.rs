pub mod client;

#[derive(Default, Clone, PartialEq)]
enum Proto {
    Http1,
    Http2,
    #[default]
    Auto
}