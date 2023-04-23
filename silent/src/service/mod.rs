use crate::conn::SilentConnection;
use crate::route::Route;
use bytes::Bytes;
use http_body_util::Full;
use hyper::service::Service;
use hyper::{body::Incoming as IncomingBody, Request, Response};
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use tokio::net::TcpListener;
// use crate::rt::RtExecutor;

pub struct Server {
    route: Option<Route>,
    addr: SocketAddr,
    conn: Arc<SilentConnection>,
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

impl Server {
    pub fn new() -> Self {
        Self {
            route: None,
            addr: ([127, 0, 0, 1], 8000).into(),
            conn: Arc::new(SilentConnection::default()),
        }
    }

    pub fn bind(&mut self, addr: SocketAddr) -> &mut Self {
        self.addr = addr;
        self
    }

    #[must_use]
    pub fn bind_route(&mut self, route: Route) -> &mut Self {
        self.route = Some(route);
        self
    }

    pub async fn serve(&self) {
        let Self { conn, .. } = self;
        println!("Listening on http://{}", self.addr);
        let listener = TcpListener::bind(self.addr).await.unwrap();
        // let conn = Arc::new(conn.clone());
        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    println!("Accepting from: {}", stream.peer_addr().unwrap());
                    let route = self.route.clone().unwrap();
                    let conn = conn.clone();
                    tokio::task::spawn(async move {
                        if let Err(err) = conn.http1.serve_connection(stream, Serve { route }).await
                        {
                            println!("Failed to serve connection: {:?}", err);
                        }
                    });
                }
                Err(e) => {
                    tracing::error!(error = ?e, "accept connection failed");
                }
            }
        }
    }

    pub fn run(&self) {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(self.serve());
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
