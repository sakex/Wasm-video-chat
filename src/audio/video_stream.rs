use wasm_bindgen::prelude::*;
use web_sys::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::future_to_promise;
use crate::js_extend::ConnectionOffer;
use crate::{get, js_await};
use std::rc::Rc;
use wasm_bindgen::__rt::std::collections::HashMap;
use wasm_bindgen::__rt::WasmRefCell;

#[derive(Serialize)]
pub struct VideoConstraints {
    width: i32,
    height: i32,
    #[serde(rename(serialize = "frameRate"))]
    frame_rate: i32,
}

#[derive(Serialize)]
pub struct StunServer {
    url: &'static str,
}

#[derive(Serialize)]
pub struct TurnServer {
    url: &'static str,
    credential: &'static str,
    username: &'static str,
}

fn create_video(muted: bool) -> Result<Rc<web_sys::HtmlVideoElement>, JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let video = document.create_element("video").unwrap().unchecked_into::<web_sys::HtmlVideoElement>();
    video.set_autoplay(true);
    video.set_muted(muted);
    video.set_width(300);
    video.set_height(300);
    video.style().set_property("transform", "rotateY(180deg)")?;
    video.style().set_property("-webkit-transform", "rotateY(180deg)")?;
    video.style().set_property("-moz-transform", "rotateY(180deg)")?;
    Ok(Rc::new(video))
}

struct Connection {
    peer: Rc<RtcPeerConnection>,
    on_ice_candidate: js_sys::Function,
    video: Rc<web_sys::HtmlVideoElement>,
    on_state_change: Closure<dyn FnMut(JsValue)>
}

impl Connection {
    fn create_config() -> RtcConfiguration {
        let mut config = RtcConfiguration::new();
        let arr = js_sys::Array::new();
        let stun = StunServer {
            url: "stun:stun.l.google.com:19302"
        };
        let turn = TurnServer {
            url: "turn:numb.viagenie.ca",
            credential: "muazkh",
            username: "webrtc@live.com",
        };
        arr.push(&JsValue::from_serde(&stun).unwrap());
        arr.push(&JsValue::from_serde(&turn).unwrap());
        config.ice_servers(&arr);
        config
    }

    fn state_change_cb(video_rc: Rc<web_sys::HtmlVideoElement>, on_state: Box<dyn Fn()>) -> Closure<dyn FnMut(JsValue)> {
        Closure::wrap(Box::new(move |event: JsValue| {
            let js_state = get![event => "target" => "iceConnectionState"];
            match js_state.as_string() {
                None => {
                    console::error_1(&event);
                    panic!("Invalid string");
                }
                Some(state) if state == "failed" || state == "closed" => {
                    console::log_1(&JsValue::from_str("in"));
                    let video: &HtmlVideoElement = video_rc.as_ref();
                    video.parent_node().unwrap().remove_child(&video).unwrap();
                    (on_state)();
                }
                Some(_) => {}
            }
        }) as Box<dyn FnMut(JsValue)>)
    }

    pub fn new(on_state: Box<dyn Fn()>) -> Connection {
        let video = create_video(false).unwrap();
        let on_state_change = Connection::state_change_cb(video.clone(), on_state);
        let config = Connection::create_config();
        let raw_peer = RtcPeerConnection::new_with_configuration(&config).unwrap();
        raw_peer.set_oniceconnectionstatechange(on_state_change.as_ref().dyn_ref());
        let peer: Rc<RtcPeerConnection> = Rc::new(raw_peer);
        Connection {
            video,
            peer,
            on_ice_candidate: js_sys::Function::new_no_args(""),
            on_state_change
        }
    }

    pub fn create_offer(&self, stream: &MediaStream) -> ConnectionOffer {
        let peer = Rc::clone(&self.peer);

        let on_track = self.track_cb();

        let cb1 = self.ice_candidate_cb();

        peer.set_onicecandidate(cb1.as_ref().dyn_ref());

        peer.set_ontrack(on_track.as_ref().dyn_ref());

        stream.get_tracks().iter().for_each(|track: JsValue| {
            peer.add_track_0(&track.unchecked_into(), &stream);
        });

        let p = future_to_promise(async move {
            let mut options: RtcOfferOptions = RtcOfferOptions::new();
            options.offer_to_receive_audio(true);
            options.offer_to_receive_video(true);
            let promise = peer.create_offer_with_rtc_offer_options(&options);

            let js_offer: JsValue = js_await![promise].unwrap();
            let offer: RtcSessionDescriptionInit = js_offer.unchecked_into();
            let set_local_promise = peer.as_ref().set_local_description(&offer);
            js_await![set_local_promise].unwrap();
            Ok(offer.unchecked_into())
        });

        let mut cb_ret = ConnectionOffer::new(p);
        cb_ret.add_cb(on_track);
        cb_ret.add_cb(cb1);
        cb_ret
    }

    pub fn accept_offer(&self, offer: RtcSessionDescriptionInit, stream: &MediaStream) -> ConnectionOffer {
        let peer = Rc::clone(&self.peer);

        let on_track = self.track_cb();

        let cb1 = self.ice_candidate_cb();
        peer.set_onicecandidate(cb1.as_ref().dyn_ref());

        peer.set_ontrack(on_track.as_ref().dyn_ref());

        stream.get_tracks().iter().for_each(|track: JsValue| {
            peer.add_track_0(&track.unchecked_into(), &stream);
        });

        let p = future_to_promise(async move {
            let set_remote_promise = peer.as_ref().set_remote_description(&offer);
            js_await![set_remote_promise].unwrap();

            let answer_promise = peer.as_ref().create_answer();
            let js_answer: JsValue = js_await![answer_promise].unwrap();
            let answer: RtcSessionDescriptionInit = js_answer.unchecked_into();
            js_await![peer.as_ref().set_local_description(&answer)].unwrap();
            Ok(answer.unchecked_into())
        });

        let mut cb_ret = ConnectionOffer::new(p);
        cb_ret.add_cb(on_track);
        cb_ret.add_cb(cb1);
        cb_ret
    }

    pub fn accept_answer(&self, answer: RtcSessionDescriptionInit) -> ConnectionOffer {
        let peer = Rc::clone(&self.peer);

        let p = future_to_promise(async move {
            let set_remote_promise = peer.as_ref().set_remote_description(&answer);
            js_await![set_remote_promise].unwrap();
            Ok(JsValue::TRUE)
        });

        let cb_ret = ConnectionOffer::new(p);
        cb_ret
    }

    fn ice_candidate_cb(&self) -> Closure<dyn FnMut(JsValue)> {
        let cb = self.on_ice_candidate.clone();
        Closure::wrap(Box::new(move |event: JsValue| {
            match get![event => "candidate"].dyn_into::<RtcIceCandidate>() {
                Ok(candidate) => {
                    match &get![candidate => "protocol"].as_string() {
                        None => {}
                        Some(proto) if proto == "udp" => {
                            cb.call1(&JsValue::NULL, &candidate).unwrap();
                        }
                        _ => {}
                    }
                }
                Err(_e) => {}
            };
        }) as Box<dyn FnMut(JsValue)>)
    }

    fn track_cb(&self) -> Closure<dyn FnMut(JsValue)> {
        let video_rc = Rc::clone(&self.video);
        Closure::wrap(Box::new(move |event: JsValue| {
            let video: &HtmlVideoElement = video_rc.as_ref();
            match video.src_object() {
                Some(_video) => {}
                None => {
                    let streams: js_sys::Array = get![event => "streams"].unchecked_into();
                    let js_stream: JsValue = streams.get(0);
                    let stream: MediaStream = js_stream.unchecked_into();
                    video.set_src_object(Some(&stream));
                }
            }
        }) as Box<dyn FnMut(JsValue)>)
    }

    pub fn set_on_ice_candidate(&mut self, closure: js_sys::Function) {
        self.on_ice_candidate = closure;
    }

    pub fn add_ice_candidate(&mut self, candidate: RtcIceCandidate) -> js_sys::Promise {
        self.peer.as_ref().add_ice_candidate_with_opt_rtc_ice_candidate(Option::from(&candidate))
    }
}

type ConnectionDict = Rc<WasmRefCell<HashMap<String, Connection>>>;

#[wasm_bindgen]
pub struct Streaming {
    dom_element: web_sys::Element,
    self_video: Rc<web_sys::HtmlVideoElement>,
    connections: ConnectionDict,
}


#[wasm_bindgen]
impl Streaming {
    #[wasm_bindgen(constructor)]
    pub fn new(dom_element: web_sys::Element) -> Streaming {
        let video = create_video(true).unwrap();
        Streaming {
            dom_element,
            self_video: video,
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
        let video_constraints = VideoConstraints { width: 300, height: 300, frame_rate: 10 };
        constraints.video(&JsValue::TRUE);
        let promise = devices.get_user_media_with_constraints(&constraints).unwrap();
        let video = Rc::clone(&self.self_video);
        self.dom_element.append_child(&video).unwrap();

        future_to_promise(async move {
            let js_stream: JsValue = match js_await![promise] {
                Ok(val) => val,
                Err(e) => panic!("{:?}", e)
            };
            match js_stream.dyn_into::<MediaStream>() {
                Ok(stream) => {
                    video.set_src_object(Some(&stream));
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
            self.dom_element.append_child(&co.video).unwrap();
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