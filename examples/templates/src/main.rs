use serde::Serialize;
use silent::prelude::*;

#[derive(Serialize)]
struct Temp {
    name: String,
}

fn main() {
    logger::fmt().with_max_level(Level::INFO).init();
    let route = Route::new("").get(|_req| async {
        let temp = Temp {
            name: "templates".to_string(),
        };
        Ok(TemplateResponse::from(("index.html".to_string(), temp)))
    });
    Server::new()
        .set_template_dir("examples/templates/templates/**/*")
        .bind_route(route)
        .run();
}
