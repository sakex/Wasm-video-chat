extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;
use web_sys::console;

mod audio;
mod js_extend;

use audio::audio_stream::Microphone;
use js_extend::Gettable;

fn cb(obj: JsValue) {
    console::log_1(&obj.get("inputBuffer"));
}

#[wasm_bindgen]
pub async fn load_mic() -> js_extend::RegisterCallback {
    let mic = Microphone::new(cb);
    mic.start_listening().await
}