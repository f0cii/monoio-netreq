use crate::request::RequestBody;
use bytes::Bytes;
use monoio_http::common::body::{FixedBody, HttpBody};

pub struct MonoioBody;
impl RequestBody for MonoioBody {
    type Body = HttpBody;

    fn create_body(bytes: Option<Bytes>) -> Self::Body {
        HttpBody::fixed_body(bytes)
    }
}
