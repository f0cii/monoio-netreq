use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};
use hyper::body::{Body as HttpBody, Frame};
use hyper::Error;
use bytes::Bytes;

#[derive(Clone)]
pub struct HyperBody {
    _marker: PhantomData<*const ()>,
    data: Option<Bytes>,
}

impl From<Bytes> for HyperBody {
    fn from(a: Bytes) -> Self {
        HyperBody {
            _marker: PhantomData,
            data: Some(a.into()),
        }
    }
}

impl HttpBody for HyperBody {
    type Data = Bytes;
    type Error = Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        Poll::Ready(self.get_mut().data.take().map(|d| Ok(Frame::data(d))))
    }
}
