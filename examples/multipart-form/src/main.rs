use serde::{Deserialize, Serialize};
use silent::prelude::*;

fn main() {
    logger::fmt().init();
    let route = Route::new("").get_html(show_form).post(accept_form);
    Server::new().bind_route(route).run();
}

async fn show_form(_req: Request) -> Result<&'static str, SilentError> {
    Ok(r#"
        <!doctype html>
        <html>
            <head></head>
            <body>
                <form action="" method="post" enctype="multipart/form-data">
                    <label>
                        Upload file:
                        <input type="file" name="files" multiple>
                    </label>

                    <input type="submit" value="Upload files">
                </form>
            </body>
        </html>
        "#)
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct File {
    name: String,
    file_name: String,
}

async fn accept_form(mut req: Request) -> Result<Vec<File>, SilentError> {
    let mut result_files = vec![];
    if let Some(files) = req.files("files").await {
        for file in files {
            result_files.push(File {
                name: file.name().unwrap_or("file").to_string(),
                file_name: file.path().to_string_lossy().to_string(),
            });
        }
    }
    Ok(result_files)
}
