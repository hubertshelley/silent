use std::net::SocketAddr;
use hyper::server::conn::http1;
use tokio::net::TcpListener;

pub struct Service {
    route: Option<String>,
    addr: SocketAddr,
}

impl Service {
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

    pub fn append(&mut self, route: String) -> &mut Self {
        self.route = Some(route);
        self
    }

    pub async fn run(&self) {
        // println!("Listening on http://{}", self.addr);
        // let listener = TcpListener::bind(self.addr).await.unwrap();
        // loop {
        //     let (stream, _) = listener.accept().await.unwrap();
        //     tokio::task::spawn(async move {
        //         if let Err(err) = http1::Builder::new()
        //             .serve_connection(stream, Svc { counter: 81818 })
        //             .await
        //         {
        //             println!("Failed to serve connection: {:?}", err);
        //         }
        //     });
        // }
    }
}
