use silent::prelude::*;

fn main() {
    logger::fmt().with_max_level(Level::INFO).init();
    let route = Route::new("").get(|mut req: Request| async move {
        let session = req.extensions_mut().get_mut::<Session>();
        println!("{:?}", session);
        match session {
            None => {}
            Some(session) => {
                if let Some(state) = session.clone().get::<i64>("state") {
                    session.insert("state", state + 1)?;
                    return Ok(state.to_string());
                } else {
                    session.insert("state", 1)?;
                }
            }
        }
        Ok("hello world".to_string())
    });
    Server::new().bind_route(route).run();
}
