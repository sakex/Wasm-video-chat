use wasm_bindgen::prelude::*;
use web_sys::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::future_to_promise;
use crate::js_extend::ConnectionOffer;
use crate::{js_await};
use std::rc::Rc;
use wasm_bindgen::__rt::std::collections::HashMap;
use wasm_bindgen::__rt::WasmRefCell;

use crate::connection_stream::connection::Connection;
use crate::connection_stream::utils::{create_video, draw_video};

#[derive(Serialize)]
pub struct VideoConstraints {
    width: i32,
    height: i32,
    #[serde(rename(serialize = "frameRate"))]
    frame_rate: i32,
}

type ConnectionDict = Rc<WasmRefCell<HashMap<String, Connection>>>;

#[wasm_bindgen]
pub struct Streaming {
    dom_element: web_sys::Element,
    self_video: Rc<web_sys::HtmlVideoElement>,
    self_canvas: Rc<web_sys::HtmlCanvasElement>,
    connections: ConnectionDict,
}


#[wasm_bindgen]
impl Streaming {
    #[wasm_bindgen(constructor)]
    pub fn new(dom_element: web_sys::Element) -> Streaming {
        let (video, canvas) = create_video(true).unwrap();
        Streaming {
            dom_element,
            self_video: video,
            self_canvas: canvas,
            connections: Rc::new(WasmRefCell::new(HashMap::new())),
        }
    }

    pub fn set_on_ice_candidate(&mut self, id: String, closure: js_sys::Function) {
        match self.connections.borrow_mut().get_mut(&id) {
            Some(connection) => { connection.set_on_ice_candidate(closure); }
            None => panic!("Id {} does not exist", &id)
        }
    }

    pub fn add_ice_candidate(&mut self, id: String, candidate: RtcIceCandidate) -> js_sys::Promise {
        match self.connections.borrow_mut().get_mut(&id) {
            Some(connection) => { connection.add_ice_candidate(candidate) }
            None => panic!("Id {} does not exist", &id)
        }
    }

    pub fn create_offer(&mut self, id: String) -> ConnectionOffer {
        //let stream = match get_canvas_stream(self.self_canvas.clone(), 20.0) {
        let stream = match self.self_video.as_ref().src_object() {
            Some(s) => s,
            None => panic!("Stream not set")
        };
        match self.connections.borrow().get(&id) {
            Some(connection) => { connection.create_offer(&stream) }
            None => panic!("Id {} does not exist", &id)
        }
    }

    pub fn accept_offer(&mut self, id: String, offer: RtcSessionDescriptionInit) -> ConnectionOffer {
        let stream = match self.self_video.as_ref().src_object() {
            Some(s) => s,
            None => panic!("Stream not set")
        };
        match self.connections.borrow_mut().get_mut(&id) {
            Some(connection) => { connection.accept_offer(offer, &stream) }
            None => panic!("Id {} does not exist", &id)
        }
    }


    pub fn accept_answer(&mut self, id: String, answer: RtcSessionDescriptionInit) -> ConnectionOffer {
        match self.connections.borrow_mut().get_mut(&id) {
            Some(connection) => { connection.accept_answer(answer) }
            None => panic!("Id {} does not exist", &id)
        }
    }

    pub fn load_video(&mut self) -> js_sys::Promise {
        let devices = web_sys::window().unwrap().navigator().media_devices().unwrap();
        let mut constraints = MediaStreamConstraints::new();
        constraints.audio(&JsValue::TRUE);
        let _video_constraints = VideoConstraints { width: 300, height: 300, frame_rate: 10 };
        constraints.video(&JsValue::TRUE);
        let promise = devices.get_user_media_with_constraints(&constraints).unwrap();
        let video = self.self_video.clone();
        let canvas = self.self_canvas.clone();

        self.dom_element.append_child(&canvas).unwrap();

        future_to_promise(async move {
            let js_stream: JsValue = match js_await![promise] {
                Ok(val) => val,
                Err(e) => panic!("{:?}", e)
            };
            match js_stream.dyn_into::<MediaStream>() {
                Ok(stream) => {
                    video.set_src_object(Some(&stream));
                    let _p = video.play().unwrap();
                    draw_video(video, canvas).unwrap();
                    Ok(stream.unchecked_into())
                }
                Err(e) => Err(e)
            }
        })
    }

    fn on_state(&mut self, id: String) -> Box<dyn Fn()> {
        let rc = self.connections.clone();
        Box::new(move || {
            let connections = &*rc;
            connections.borrow_mut().remove(&id);
        })
    }

    pub fn create_connection(&mut self, id: String) -> Result<JsValue, JsValue> {
        if !self.connections.borrow().contains_key(&id) {
            let co = Connection::new(self.on_state(id.clone()));
            self.dom_element.append_child(&co.get_canvas()).unwrap();
            self.connections.borrow_mut().insert(id, co);
            return Ok(JsValue::TRUE);
        }
        Err(JsValue::from_str("Id already created"))
    }


    pub fn get_ids(&mut self) -> js_sys::Set {
        let set: js_sys::Set = js_sys::Set::new(&JsValue::UNDEFINED);
        for key in self.connections.borrow_mut().keys() {
            set.add(&JsValue::from_str(key));
        }
        set
    }
}