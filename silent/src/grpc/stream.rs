use std::task::Poll;

use axum::body::Body as AxumBody;
use bytes::Bytes;
use futures::Stream;
use http::response::Parts;
use http_body::Body;
use pin_project_lite::pin_project;

pin_project! {
    pub(crate) struct GrpcStream{
        #[pin]
        body: AxumBody,
        parts: Parts,
    }
}

impl GrpcStream {
    pub fn new(body: AxumBody, parts: Parts) -> Self {
        Self { body, parts }
    }
}
fn title_case(dst: &mut Vec<u8>, name: &[u8]) {
    dst.reserve(name.len());

    // Ensure first character is uppercased
    let mut prev = b'-';
    for &(mut c) in name {
        if prev == b'-' {
            c.make_ascii_uppercase();
        }
        dst.push(c);
        prev = c;
    }
}
#[inline]
fn extend(dst: &mut Vec<u8>, data: &[u8]) {
    dst.extend_from_slice(data);
}
impl Stream for GrpcStream {
    type Item = crate::Result<Bytes>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let mut stream = self.project();
        match stream.body.as_mut().poll_frame(cx) {
            Poll::Ready(Some(Ok(chunk))) => match chunk.into_data() {
                Ok(chunk) => Poll::Ready(Some(Ok(chunk))),
                Err(_e) => {
                    Poll::Ready(None)
                    // error!("Failed to convert chunk to bytes: {:?}", e);
                    // let mut buf = Vec::new();
                    // if e.is_trailers() {
                    //     let header_map = HeaderMap::with_capacity(3 + stream.parts.headers.len());
                    //     let mut trailers = e.into_trailers().unwrap();
                    //     trailers.extend(header_map);
                    //     for (name, value) in trailers.iter() {
                    //         title_case(&mut buf, name.as_str().as_bytes());
                    //         extend(&mut buf, b": ");
                    //         extend(&mut buf, value.as_bytes());
                    //         extend(&mut buf, b"\r\n");
                    //     }
                    // };
                    // if buf.is_empty() {
                    //     return Poll::Ready(None);
                    // }
                    // let mut trailer = Bytes::from_static(b"0\r\n").to_vec();
                    // trailer.extend_from_slice(buf.as_slice());
                    // trailer.extend_from_slice(b"\r\n");
                    // Poll::Ready(Some(Ok(Bytes::from(trailer))))
                }
            },
            Poll::Pending => Poll::Pending,
            _ => Poll::Ready(None),
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let size_hint = self.body.size_hint();
        (
            size_hint.lower() as usize,
            size_hint.upper().map(|x| x as usize),
        )
    }
}
