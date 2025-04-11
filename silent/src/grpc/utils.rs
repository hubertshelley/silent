use crate::Response;
use crate::prelude::ResBody;
use http::response::Parts;
use http_body_util::BodyExt;
use tonic::body::Body;

#[cfg(feature = "grpc")]
/// 合并axum响应
#[inline]
pub async fn merge_grpc_response(res: &mut Response, grpc_res: http::Response<Body>) {
    let (parts, body) = grpc_res.into_parts();
    let Parts {
        status,
        headers,
        extensions,
        version,
        ..
    } = parts;
    res.status = status;
    res.version = version;
    res.headers.extend(headers);
    res.extensions.extend(extensions);
    res.body = ResBody::Boxed(Box::pin(body.map_err(|e| e.into())));
}
