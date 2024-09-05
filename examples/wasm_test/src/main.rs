use crossbeam_epoch::Pointable;
use extism::*;
use silent::prelude::{HandlerAppend, Route};
use silent::{Handler, Request};
use std::error::Error;
use std::pin::Pin;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let wasm = Wasm::file("./examples/wasm/target/wasm32-wasip1/release/examples_wasm.wasm");
    let manifest = Manifest::new([wasm]);
    let mut plugin = Plugin::new(&manifest, [], true)?;
    let ptr: u64 = plugin
        .call("get_route", ())
        .expect("Failed to call get_route");
    println!("{:?}", ptr);
    // let route = Route::new("111").get(|_req| async { Ok("hello world") });
    // let ptr = &route as *const _ as usize;
    let values = unsafe { Box::<Pin<String>>::deref(ptr as usize) };
    let a = values.to_string();
    println!("values: {}", a);
    // println!("values: {:?}", values);
    // let res = values.call(Request::empty()).await;
    // println!("result: {:?}", res);
    Ok(())
}
