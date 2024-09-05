use extism_pdk::{plugin_fn, FnResult};
use silent::prelude::{HandlerAppend, Route};

#[plugin_fn]
pub fn get_route() -> FnResult<u64> {
    // let route = Route::new("hello").get(|_req| async { Ok("hello world") });
    let route = Box::pin("hello world".to_string());
    Ok(&route as *const _ as u64)
}
