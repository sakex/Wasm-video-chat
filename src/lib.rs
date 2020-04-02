extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;
use web_sys::*;
use wasm_bindgen::JsCast;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}


#[wasm_bindgen]
pub struct RecordHandle {
    cb: Closure<dyn FnMut(JsValue)>
}

impl RecordHandle {
    pub fn new(cb: Closure<dyn FnMut(JsValue)>) -> RecordHandle {
        RecordHandle { cb }
    }
}

fn start_listener_callback(stream: MediaStream) -> Closure<dyn FnMut(JsValue)> {
    let context: AudioContext = AudioContext::new().unwrap();
    let source: MediaStreamAudioSourceNode = context.create_media_stream_source(&stream).unwrap();
    let processor: ScriptProcessorNode = context.create_script_processor().unwrap();

    source.connect_with_audio_node(&processor);
    processor.connect_with_audio_node(&context.destination());

    let listener = Closure::wrap(Box::new(|js_stream: JsValue| {
        log("hi");
    }) as Box<dyn FnMut(JsValue)>);

    processor.set_onaudioprocess(listener.as_ref().dyn_ref());

    listener
}

#[wasm_bindgen]
pub fn load_mic() -> RecordHandle {
    let devices = web_sys::window().unwrap().navigator().media_devices().unwrap();
    let mut constraints = MediaStreamConstraints::new();
    constraints.audio(&JsValue::TRUE);
    let promise = devices.get_user_media_with_constraints(&constraints).unwrap();

    let cb = Closure::wrap(Box::new(|js_stream: JsValue| {
        let stream: MediaStream = js_stream.unchecked_into();
        start_listener(stream, &listener);
    }) as Box<dyn FnMut(JsValue)>);

    promise.then(&cb);
    RecordHandle::new(cb)
}