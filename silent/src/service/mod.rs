use crate::route::Route;
use bytes::Bytes;
use http_body_util::Full;
use hyper::server::conn::http1;
use hyper::service::Service;
use hyper::{body::Incoming as IncomingBody, Request, Response};
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use tokio::net::TcpListener;

pub struct Server {
    route: Option<Route>,
    addr: SocketAddr,
}

impl Server {
    pub fn new() -> Self {
        Self {
            route: None,
            addr: ([127, 0, 0, 1], 8000).into(),
        }
    }

    pub fn bind(&mut self, addr: SocketAddr) -> &mut Self {
        self.addr = addr;
        self
    }

    #[must_use]
    pub fn append(&mut self, route: Route) -> &mut Self {
        self.route = Some(route);
        self
    }

    pub async fn run(&self) {
        println!("Listening on http://{}", self.addr);
        let listener = TcpListener::bind(self.addr).await.unwrap();
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            let route = self.route.clone().unwrap();
            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .serve_connection(stream, Serve { route })
                    .await
                {
                    println!("Failed to serve connection: {:?}", err);
                }
            });
        }
    }
}

struct Serve {
    route: Route,
}

impl From<Route> for Serve {
    fn from(route: Route) -> Self {
        Self { route }
    }
}

impl Service<Request<IncomingBody>> for Serve {
    type Response = Response<Full<Bytes>>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&mut self, req: Request<IncomingBody>) -> Self::Future {
        fn mk_response(s: String) -> Result<Response<Full<Bytes>>, hyper::Error> {
            Ok(Response::builder().body(Full::new(Bytes::from(s))).unwrap())
        }

        println!("req: {:?}", self.route);

        let res = match req.uri().path() {
            "/" => mk_response(format!("home! counter = {:?}", 1)),
            "/posts" => mk_response(format!("posts, of course! counter = {:?}", 1)),
            "/authors" => mk_response(format!("authors extraordinare! counter = {:?}", 1)),
            // Return the 404 Not Found for other routes, and don't increment counter.
            _ => return Box::pin(async { mk_response("oh no! not found".into()) }),
        };

        Box::pin(async { res })
    }
}
