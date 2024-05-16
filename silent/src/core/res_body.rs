use crate::error::BoxedError;
use bytes::Bytes;
use futures_util::stream::{BoxStream, Stream};
use futures_util::TryStreamExt;
use http_body::{Body, Frame, SizeHint};
use hyper::body::Incoming;
use std::collections::VecDeque;
use std::error::Error as StdError;
use std::io::{Error as IoError, ErrorKind};
use std::pin::Pin;
use std::task::{self, Context, Poll};

/// 响应体
pub enum ResBody {
    /// None body.
    None,
    /// Once bytes body.
    Once(Bytes),
    /// Chunks body.
    Chunks(VecDeque<Bytes>),
    /// Incoming default body.
    Incoming(Incoming),
    /// Stream body.
    Stream(BoxStream<'static, Result<Bytes, BoxedError>>),
    /// Boxed body.
    Boxed(Pin<Box<dyn Body<Data = Bytes, Error = BoxedError> + Send + Sync + 'static>>),
}

/// 转换数据为响应Body
pub fn full<T: Into<Bytes>>(chunk: T) -> ResBody {
    ResBody::Once(chunk.into())
}

/// 转换数据为响应Body
pub fn stream_body<S, O, E>(stream: S) -> ResBody
where
    S: Stream<Item = Result<O, E>> + Send + 'static,
    O: Into<Bytes> + 'static,
    E: Into<Box<dyn StdError + Send + Sync>> + 'static,
{
    let mapped = stream.map_ok(Into::into).map_err(Into::into);
    ResBody::Stream(Box::pin(mapped))
}

impl Stream for ResBody {
    type Item = Result<Bytes, IoError>;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Option<Self::Item>> {
        match self.get_mut() {
            ResBody::None => Poll::Ready(None),
            ResBody::Once(bytes) => {
                if bytes.is_empty() {
                    Poll::Ready(None)
                } else {
                    let bytes = std::mem::replace(bytes, Bytes::new());
                    Poll::Ready(Some(Ok(bytes)))
                }
            }
            ResBody::Chunks(chunks) => Poll::Ready(chunks.pop_front().map(Ok)),
            ResBody::Incoming(body) => match Body::poll_frame(Pin::new(body), cx) {
                Poll::Ready(Some(Ok(frame))) => Poll::Ready(frame.into_data().map(Ok).ok()),
                Poll::Ready(Some(Err(e))) => {
                    Poll::Ready(Some(Err(IoError::new(ErrorKind::Other, e))))
                }
                Poll::Ready(None) => Poll::Ready(None),
                Poll::Pending => Poll::Pending,
            },
            ResBody::Stream(stream) => stream
                .as_mut()
                .poll_next(cx)
                .map_err(|e| IoError::new(ErrorKind::Other, e)),
            ResBody::Boxed(body) => match Body::poll_frame(Pin::new(body), cx) {
                Poll::Ready(Some(Ok(frame))) => Poll::Ready(frame.into_data().map(Ok).ok()),
                Poll::Ready(Some(Err(e))) => {
                    Poll::Ready(Some(Err(IoError::new(ErrorKind::Other, e))))
                }
                Poll::Ready(None) => Poll::Ready(None),
                Poll::Pending => Poll::Pending,
            },
        }
    }
}

impl Body for ResBody {
    type Data = Bytes;
    type Error = IoError;

    fn poll_frame(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        match self.poll_next(_cx) {
            Poll::Ready(Some(Ok(bytes))) => Poll::Ready(Some(Ok(Frame::data(bytes)))),
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }

    fn is_end_stream(&self) -> bool {
        match self {
            ResBody::None => true,
            ResBody::Once(bytes) => bytes.is_empty(),
            ResBody::Chunks(chunks) => chunks.is_empty(),
            ResBody::Incoming(body) => body.is_end_stream(),
            ResBody::Boxed(body) => body.is_end_stream(),
            ResBody::Stream(_) => false,
        }
    }

    fn size_hint(&self) -> SizeHint {
        match self {
            ResBody::None => SizeHint::with_exact(0),
            ResBody::Once(bytes) => SizeHint::with_exact(bytes.len() as u64),
            ResBody::Chunks(chunks) => {
                let size = chunks.iter().map(|bytes| bytes.len() as u64).sum();
                SizeHint::with_exact(size)
            }
            ResBody::Incoming(recv) => recv.size_hint(),
            ResBody::Boxed(recv) => recv.size_hint(),
            ResBody::Stream(_) => SizeHint::default(),
        }
    }
}

impl From<Bytes> for ResBody {
    fn from(value: Bytes) -> ResBody {
        ResBody::Once(value)
    }
}

impl From<Incoming> for ResBody {
    fn from(value: Incoming) -> ResBody {
        ResBody::Incoming(value)
    }
}

impl From<String> for ResBody {
    #[inline]
    fn from(value: String) -> ResBody {
        ResBody::Once(value.into())
    }
}

impl From<&'static [u8]> for ResBody {
    fn from(value: &'static [u8]) -> ResBody {
        ResBody::Once(value.into())
    }
}

impl From<&'static str> for ResBody {
    fn from(value: &'static str) -> ResBody {
        ResBody::Once(value.into())
    }
}

impl From<Vec<u8>> for ResBody {
    fn from(value: Vec<u8>) -> ResBody {
        ResBody::Once(value.into())
    }
}

impl From<Box<[u8]>> for ResBody {
    fn from(value: Box<[u8]>) -> ResBody {
        ResBody::Once(value.into())
    }
}
