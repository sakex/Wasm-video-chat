use wasm_bindgen::prelude::*;
use web_sys::*;
use wasm_bindgen::JsCast;
use crate::js_extend::js_extension;

type Callback = fn(JsValue);

pub struct Microphone {
    callback: Callback,
}

impl Microphone {
    pub fn new(callback: Callback) -> Microphone {
        Microphone{callback}
    }

    pub async fn start_listening(&self) -> js_extension::RegisterCallback {
        let devices = web_sys::window().unwrap().navigator().media_devices().unwrap();
        let mut constraints = MediaStreamConstraints::new();
        constraints.audio(&JsValue::TRUE);
        let promise = devices.get_user_media_with_constraints(&constraints).unwrap();

        let js_stream: JsValue = wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();

        let stream: MediaStream = js_stream.unchecked_into();
        self.start_listener(stream)
    }

    fn start_listener(&self, stream: MediaStream) -> js_extension::RegisterCallback {
        let context: AudioContext = AudioContext::new().unwrap();
        let source: MediaStreamAudioSourceNode = context.create_media_stream_source(&stream).unwrap();
        let processor: ScriptProcessorNode = context.create_script_processor_with_buffer_size(1024).unwrap();

        source.connect_with_audio_node(&processor);
        processor.connect_with_audio_node(&context.destination());

        let listener = Closure::wrap(Box::new(self.callback) as Box<dyn FnMut(JsValue)>);

        processor.set_onaudioprocess(listener.as_ref().dyn_ref());

        js_extension::RegisterCallback::new(listener)
    }
}
