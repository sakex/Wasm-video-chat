use wasm_bindgen::prelude::*;
use web_sys::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::future_to_promise;
use crate::js_extend::ConnectionOffer;
use crate::{js_await};
use std::rc::Rc;
use crate::connection_stream::utils::{create_video, draw_video};

#[derive(Serialize)]
pub struct StunServer {
    urls: Vec<&'static str>,
}

#[derive(Serialize)]
pub struct TurnServer {
    urls: Vec<&'static str>,
    credential: &'static str,
    username: &'static str,
}

pub struct Connection {
    peer: Rc<RtcPeerConnection>,
    on_ice_candidate: js_sys::Function,
    video: Rc<web_sys::HtmlVideoElement>,
    canvas: Rc<web_sys::HtmlCanvasElement>,
    on_state_change: Closure<dyn FnMut(JsValue)>
}

impl Connection {
    pub fn get_canvas(&self) -> Rc<web_sys::HtmlCanvasElement> {
        self.canvas.clone()
    }

    fn create_config() -> RtcConfiguration {
        let mut config = RtcConfiguration::new();
        let arr = js_sys::Array::new();
        let stun = StunServer {
            urls: vec!["stun:stun.l.google.com:19302"]
        };
        let turn = TurnServer {
            urls: vec!["turn:numb.viagenie.ca"],
            credential: "muazkh",
            username: "webrtc@live.com",
        };
        arr.push(&JsValue::from_serde(&stun).unwrap());
        arr.push(&JsValue::from_serde(&turn).unwrap());
        config.ice_servers(&arr);
        config
    }

    fn state_change_cb(canvas_rc: Rc<web_sys::HtmlCanvasElement>, on_state: Box<dyn Fn()>) -> Closure<dyn FnMut(JsValue)> {
        Closure::wrap(Box::new(move |event: JsValue| {
            let js_state = get![event => "target" => "iceConnectionState"];
            match js_state.as_string() {
                None => {
                    console::error_1(&event);
                    panic!("Invalid string");
                }
                Some(state) if state == "failed" || state == "disconnected" || state == "closed" => {
                    console::log_1(&JsValue::from_str("in"));
                    let canvas: &HtmlCanvasElement = canvas_rc.as_ref();
                    let parent = canvas.parent_element().unwrap();
                    parent.remove_child(&canvas).unwrap();
                    (on_state)();
                }
                Some(_) => {}
            }
        }) as Box<dyn FnMut(JsValue)>)
    }

    pub fn new(on_state: Box<dyn Fn()>) -> Connection {
        let (video, canvas) = create_video(false).unwrap();
        let on_state_change = Connection::state_change_cb(canvas.clone(), on_state);
        let config = Connection::create_config();
        let raw_peer = RtcPeerConnection::new_with_configuration(&config).unwrap();
        raw_peer.set_oniceconnectionstatechange(on_state_change.as_ref().dyn_ref());
        let peer: Rc<RtcPeerConnection> = Rc::new(raw_peer);
        Connection {
            video,
            canvas,
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
        let canvas_rc = Rc::clone(&self.canvas);
        Closure::wrap(Box::new(move |event: JsValue| {
            match video_rc.as_ref().src_object() {
                Some(_src) => {}
                None => {
                    let streams: js_sys::Array = get![event => "streams"].unchecked_into();
                    let js_stream: JsValue = streams.get(0);
                    let stream: MediaStream = js_stream.unchecked_into();
                    video_rc.set_src_object(Some(&stream));
                    let _ = video_rc.play().unwrap();
                    draw_video(video_rc.clone(), canvas_rc.clone()).unwrap();
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
