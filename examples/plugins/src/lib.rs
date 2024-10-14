use silent::prelude::{HandlerAppend, Route};
#[no_mangle]
#[allow(improper_ctypes_definitions)]
extern "C" fn get_route() -> Route {
    Route::new("hello").get(|_req| async { Ok("hello world") })
}
