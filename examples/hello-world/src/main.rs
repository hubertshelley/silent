use silent::{logger, Server};

fn main() {
    logger::fmt::init();
    Server::new().run();
}
