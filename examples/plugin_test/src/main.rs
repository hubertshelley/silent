// use crossbeam_epoch::Pointable;
use dlopen2::wrapper::{Container, WrapperApi};
use silent::prelude::{Level, Route, Server, logger};
use std::error::Error;

#[derive(WrapperApi)]
struct PluginApi {
    // plugin_metadata: unsafe extern "C" fn() -> PluginMetadata,
    #[allow(improper_ctypes_definitions)]
    get_route: unsafe extern "C" fn() -> Route,
}

fn main() -> Result<(), Box<dyn Error>> {
    logger::fmt().with_max_level(Level::INFO).init();
    let lib_path = "./libexamples_plugins.dylib";
    let container: Container<PluginApi> =
        unsafe { Container::load(lib_path) }.expect("无法加载插件");
    let route = unsafe { container.get_route() };
    Server::new().run(route);
    Ok(())
}
