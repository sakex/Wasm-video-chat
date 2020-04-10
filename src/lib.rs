extern crate wasm_bindgen;
extern crate console_error_panic_hook;

#[macro_use]
extern crate serde_derive;


mod audio;

#[macro_use]
mod js_extend;

use wasm_bindgen::prelude::*;
use js_extend::log;

#[wasm_bindgen]
pub fn init_panic_hook() {
    log("Using wasm video rtc version 0.0.1");
    console_error_panic_hook::set_once();
}