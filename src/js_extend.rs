pub mod js_extension {
    use wasm_bindgen::prelude::*;

    pub trait Gettable {
        fn get(&self, value: &str) -> JsValue;
    }

    impl Gettable for JsValue {
        fn get(&self, value: &str) -> JsValue {
            js_sys::Reflect::get(&self, &JsValue::from_str(value)).unwrap()
        }
    }

    #[wasm_bindgen]
    pub struct RegisterCallback {
        cb: Closure<dyn FnMut(JsValue)>
    }

    impl RegisterCallback {
        pub fn new(cb: Closure<dyn FnMut(JsValue)>) -> RegisterCallback {
            RegisterCallback { cb }
        }
    }

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = console)]
        pub fn log(s: &str);
    }
}