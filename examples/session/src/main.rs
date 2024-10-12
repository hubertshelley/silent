use silent::prelude::*;

fn main() {
    logger::fmt().with_max_level(Level::INFO).init();
    let route = Route::new("").get(|mut req: Request| async move {
        let state = req.session::<&str, i64>("state");
        let sessions_mut = req.sessions_mut();
        if let Some(state) = state {
            sessions_mut.insert("state", state + 1)?;
            return Ok(state.to_string());
        } else {
            sessions_mut.insert("state", 1)?;
        }
        Ok("hello world".to_string())
    });
    Server::new().run(route);
}
