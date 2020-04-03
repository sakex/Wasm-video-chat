extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

mod audio;
mod js_extend;

use js_extend::js_extension;
use audio::audio_stream::Microphone;
use js_extension::log;

fn cb(obj: JsValue) {
    log("test");
}

#[wasm_bindgen]
pub async fn load_mic() -> js_extension::RegisterCallback {
    let mic = Microphone::new(cb);
    mic.start_listening().await
}