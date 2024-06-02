use crate::prelude::ResBody;
use crate::Response;
use http::response::Parts;
use http_body_util::BodyExt;

#[cfg(feature = "grpc")]
/// 合并axum响应
#[inline]
pub async fn merge_axum(res: &mut Response, axum_res: axum::response::Response) {
    let (parts, body) = axum_res.into_parts();
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
