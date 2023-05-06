use silent::prelude::*;
use std::sync::Arc;

fn main() {
    logger::fmt().init();
    if !std::path::Path::new("static").is_dir() {
        std::fs::create_dir("static").unwrap();
        std::fs::write(
            "./static/index.html",
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Silent</title>
</head>
<body>

<h1>我的第一个标题</h1>

<p>我的第一个段落。</p>

</body>
</html>"#,
        )
        .unwrap();
    }
    let mut route = Route::new("<path:**>");
    route
        .get_handler_mut()
        .insert(Method::GET, Arc::new(static_handler("static")));
    Server::new().bind_route(route).run();
}
