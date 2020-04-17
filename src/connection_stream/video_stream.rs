use wasm_bindgen::prelude::*;
use web_sys::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::future_to_promise;
use crate::js_extend::ConnectionOffer;
use crate::js_await;
use std::rc::Rc;
use wasm_bindgen::__rt::std::collections::HashMap;
use wasm_bindgen::__rt::WasmRefCell;

use crate::connection_stream::connection::Connection;
use crate::connection_stream::render_video::{create_video, VideoRenderer};
use wasm_bindgen::__rt::core::cell::RefCell;

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
    canvas: Rc<web_sys::HtmlCanvasElement>,
    connections: ConnectionDict,
    renderer: Rc<RefCell<VideoRenderer>>,
}


#[wasm_bindgen]
impl Streaming {
    #[wasm_bindgen(constructor)]
    pub fn new(dom_element: web_sys::Element) -> Streaming {
        let video = create_video(true).unwrap();
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.create_element("canvas").unwrap().unchecked_into::<HtmlCanvasElement>();
        canvas.set_width(1100);
        canvas.set_height(726);
        let canvas_rc = Rc::new(canvas);
        Streaming {
            dom_element,
            self_video: video,
            canvas: canvas_rc.clone(),
            connections: Rc::new(WasmRefCell::new(HashMap::new())),
            renderer: Rc::new(RefCell::new(VideoRenderer::new(canvas_rc).unwrap())),
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

    pub fn load_video(&mut self) -> Result<js_sys::Promise, JsValue> {
        let devices = web_sys::window().unwrap().navigator().media_devices()?;
        let mut constraints = MediaStreamConstraints::new();
        constraints.audio(&JsValue::TRUE);
        let _video_constraints = VideoConstraints { width: 300, height: 300, frame_rate: 10 };
        constraints.video(&JsValue::TRUE);
        let promise = devices.get_user_media_with_constraints(&constraints)?;
        let video = self.self_video.clone();
        let canvas = self.canvas.clone();

        let mut renderer = self.renderer.borrow_mut();
        self.dom_element.append_child(&canvas)?;
        renderer.add_video("self".to_string(), video.clone());
        renderer.start()?;

        Ok(future_to_promise(async move {
            let js_stream: JsValue = match js_await![promise] {
                Ok(val) => val,
                Err(e) => panic!("{:?}", e)
            };
            match js_stream.dyn_into::<MediaStream>() {
                Ok(stream) => {
                    video.set_src_object(Some(&stream));
                    let _p = video.play().unwrap();
                    Ok(stream.unchecked_into())
                }
                Err(e) => Err(e)
            }
        }))
    }

    fn on_state(&mut self, id: String) -> Box<dyn Fn()> {
        let rc = self.connections.clone();
        let renderer = self.renderer.clone();
        Box::new(move || {
            let connections = &*rc;
            if let Some(_connection) = connections.borrow_mut().remove(&id) {
                renderer.borrow_mut().remove_video(&id);
            }
        })
    }

    pub fn create_connection(&mut self, id: String) -> Result<JsValue, JsValue> {
        if !self.connections.borrow().contains_key(&id) {
            let co = Connection::new(id.clone(), &mut *self.renderer.clone().borrow_mut(), self.on_state(id.clone()));
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

    pub fn not_managed(&mut self) {
        self.renderer.borrow_mut().not_managed();
    }

    pub fn set_video_pos(&mut self, id: String, x: f64, y: f64) -> Result<JsValue, JsValue> {
        self.renderer.borrow_mut().set_video_pos(id, x, y)
    }

    pub fn set_dims(&mut self, x: f64, y: f64) {
        self.renderer.borrow_mut().set_dims(x, y);
    }
}