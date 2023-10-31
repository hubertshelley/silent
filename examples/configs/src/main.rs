use silent::prelude::*;

fn main() {
    logger::fmt().with_max_level(Level::INFO).init();
    let mut configs = Configs::default();
    configs.insert(1i32);
    let route = Route::new("").get(|req| async move {
        let num = req.configs().get::<i32>().unwrap();
        Ok(*num)
    });
    Server::new().with_configs(configs).run(route);
}
