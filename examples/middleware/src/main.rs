use async_trait::async_trait;
use silent::prelude::*;
use std::sync::atomic::AtomicUsize;

fn main() {
    logger::fmt().init();
    let middle_ware = MiddleWare {
        count: AtomicUsize::new(0),
    };
    let route = Route::new("")
        .hook(middle_ware)
        .get(|_req| async { Ok("Hello World") });
    Server::new().bind_route(route).run();
}

struct MiddleWare {
    count: AtomicUsize,
}

#[async_trait]
impl MiddleWareHandler for MiddleWare {
    async fn pre_request(&self, _req: &mut Request, _res: &mut Response) -> Result<()> {
        self.count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let count = self.count.load(std::sync::atomic::Ordering::SeqCst);
        info!("pre_request count: {}", count);
        if count % 2 == 0 {
            error!("set pre_request error");
            return Err(SilentError::BusinessError {
                code: StatusCode::BAD_REQUEST,
                msg: "bad request:pre_request".to_string(),
            });
        }
        Ok(())
    }
    async fn after_response(&self, _res: &mut Response) -> Result<()> {
        let count = self.count.load(std::sync::atomic::Ordering::SeqCst);
        info!("after_response count: {}", count);
        if count % 3 == 0 {
            error!("set after_response error");
            return Err(SilentError::BusinessError {
                code: StatusCode::BAD_REQUEST,
                msg: "bad request:after_response".to_string(),
            });
        }
        Ok(())
    }
}
