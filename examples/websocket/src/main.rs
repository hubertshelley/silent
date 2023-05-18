use silent::prelude::*;

fn main() {
    logger::fmt().init();
    let route = Route::new("")
        .get(show_form)
        .append(Route::new("ws").ws(None, |_| async {}));
    Server::new().bind_route(route).run();
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
