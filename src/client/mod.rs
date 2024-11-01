pub mod client;

#[derive(Default, Clone)]
enum Proto {
    Http1,
    Http2,
    #[default]
    Auto
}