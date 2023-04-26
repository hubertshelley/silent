use crate::core::request::Request;
use crate::core::response::Response;
use crate::error::SilentError;
use async_trait::async_trait;

#[async_trait]
pub trait Handler: Send + Sync + 'static {
    async fn call(&self, req: Request) -> Result<Response, SilentError> {
        let _ = req;
        let data = vec!["foo", "bar"];
        let json = serde_json::to_string(&data).unwrap();
        Ok(Response::from(json))
    }
}

// impl Handler {
//     pub(crate) async fn call(&self, req: Request) -> Result<Response, SilentError> {
//         let _ = req;
//         let data = vec!["foo", "bar"];
//         let json = serde_json::to_string(&data).unwrap();
//         Ok(Response::from(json))
//     }
// }
