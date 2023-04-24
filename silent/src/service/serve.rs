use crate::core::res_body::ResBody;
use crate::core::response::Response;
use crate::route::Routes;
use hyper::service::Service;
use hyper::{body::Incoming as IncomingBody, Request, Response as HyperResponse};
use std::future::Future;
use std::pin::Pin;

pub(crate) struct Serve {
    pub(crate) routes: Routes,
}

impl Service<Request<IncomingBody>> for Serve {
    type Response = HyperResponse<ResBody>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&mut self, req: Request<IncomingBody>) -> Self::Future {
        tracing::info!("req: {:?}", self.routes);

        let res = match req.uri().path() {
            "/" => Response::from(format!("home! counter = {:?}", 1)),
            "/posts" => Response::from(format!("posts, of course! counter = {:?}", 1)),
            "/authors" => Response::from(format!("authors extraordinare! counter = {:?}", 1)),
            // Return the 404 Not Found for other routes, and don't increment counter.
            _ => return Box::pin(async { Ok(Response::from("oh no! not found").res) }),
        };

        Box::pin(async { Ok(res.res) })
    }
}
