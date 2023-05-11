use pyo3::prelude::*;
use silent::prelude::*;
use std::sync::Mutex;

#[pyclass]
struct SilentServer {
    server: Box<Mutex<Server>>,
}

#[pymethods]
impl SilentServer {
    #[new]
    pub fn new(host: Option<String>, port: Option<u16>) -> Self {
        let host = host.unwrap_or_else(|| "127.0.0.1".to_string());
        let port = port.unwrap_or(8000);
        let server = Self {
            server: Box::new(Mutex::new(Server::new())),
        };
        server
            .server
            .lock()
            .unwrap()
            .bind(format!("{}:{}", host, port).parse().unwrap());
        server
    }

    pub fn set_logger(&self, level: Option<String>) {
        let level = match level {
            Some(level) => match level.as_str() {
                "debug" => Level::DEBUG,
                "info" => Level::INFO,
                "warn" => Level::WARN,
                "error" => Level::ERROR,
                _ => Level::INFO,
            },
            None => Level::INFO,
        };
        logger::fmt().with_max_level(level).init();
    }

    pub fn run(&self) {
        self.server.lock().unwrap().run();
    }
}

#[pymodule]
fn silent_py(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<SilentServer>()?;

    Ok(())
}
