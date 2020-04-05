use wasm_bindgen::prelude::*;

#[macro_export]
macro_rules! get {
    ($obj: ident => $key: expr) => (js_sys::Reflect::get(&$obj, &JsValue::from_str($key)).unwrap())
}

#[macro_export]
macro_rules! js_await {
    ($promise: ident) => (wasm_bindgen_futures::JsFuture::from($promise).await.unwrap());
    ($promise: expr) => (wasm_bindgen_futures::JsFuture::from($promise).await.unwrap())
}


#[wasm_bindgen]
pub struct ConnectionOffer {
    callbacks: Vec<Closure<dyn FnMut(JsValue)>>,
    promise: js_sys::Promise,
}

#[wasm_bindgen]
impl ConnectionOffer {
    pub fn get_offer(&self) -> js_sys::Promise {
        self.promise.clone()
    }
}

impl ConnectionOffer {
    pub fn new(promise: js_sys::Promise) -> ConnectionOffer {
        ConnectionOffer { callbacks: vec![], promise }
    }

    pub fn add_cb(&mut self, cb: Closure<dyn FnMut(JsValue)>) {
        self.callbacks.push(cb);
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}
