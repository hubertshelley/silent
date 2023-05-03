use serde::{Deserialize, Serialize};
use silent::prelude::*;

fn main() {
    logger::fmt().init();
    let route = Route::new("").get_html(show_form).post(accept_form);
    Server::new().bind_route(route).run();
}

#[derive(Deserialize, Serialize, Debug)]
#[allow(dead_code)]
struct Input {
    name: String,
    email: String,
}

async fn accept_form(req: Request) -> Result<Option<Input>, SilentError> {
    req.body_parse().await
}

async fn show_form(_req: Request) -> Result<&'static str, SilentError> {
    Ok(r#"
        <!doctype html>
        <html>
            <head></head>
            <body>
                <form action="/" method="post">
                    <label for="name">
                        Enter your name:
                        <input type="text" name="name">
                    </label>

                    <label>
                        Enter your email:
                        <input type="text" name="email">
                    </label>

                    <input type="submit" value="Subscribe!">
                </form>
            </body>
        </html>
        "#)
}
