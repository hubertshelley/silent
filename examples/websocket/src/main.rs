use async_trait::async_trait;
use silent::prelude::*;

fn main() {
    logger::fmt().init();
    let route = Route::new("").get(show_form).append(
        Route::new("ws"), // .insert_handler(
                          //     Method::GET,
                          //     WSHandler {},
                          // )
    );
    Server::new().bind_route(route).run();
}

struct WSHandler {}

#[async_trait]
impl Handler for WSHandler {
    async fn call(&self, _req: Request) -> Result<Response> {
        Ok(Response::empty())
    }
}

async fn show_form(_req: Request) -> Result<&'static str> {
    Ok(r#"<!DOCTYPE html>
<html>
    <head>
        <title>WS</title>
    </head>
    <body>
        <h1>WS</h1>
        <div id="status">
            <p><em>Connecting...</em></p>
        </div>
        <script>
            const status = document.getElementById('status');
            const msg = document.getElementById('msg');
            const submit = document.getElementById('submit');
            const ws = new WebSocket(`ws://${location.host}/ws?id=123&name=dddf`);

            ws.onopen = function() {
                status.innerHTML = '<p><em>Connected!</em></p>';
            };
        </script>
    </body>
</html>
"#)
}
