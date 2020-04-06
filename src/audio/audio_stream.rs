use wasm_bindgen::prelude::*;
use web_sys::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::future_to_promise;
use std::rc::Rc;
use crate::js_extend::{log, ConnectionOffer};
use crate::{get, set, js_await};
use std::sync::Arc;


#[wasm_bindgen]
pub struct Streaming {
    dom_element: web_sys::Element,
    video1: Rc<web_sys::HtmlVideoElement>,
    video2: Rc<web_sys::HtmlVideoElement>,
    peer: Arc<RtcPeerConnection>,
    on_ice_candidate: js_sys::Function,
}

#[wasm_bindgen]
impl Streaming {
    fn create_muted_video(muted: bool) -> web_sys::HtmlVideoElement {
        let document = web_sys::window().unwrap().document().unwrap();
        let video = document.create_element("video").unwrap().unchecked_into::<web_sys::HtmlVideoElement>();
        video.set_autoplay(true);
        video.set_muted(muted);
        video.set_width(300);
        video.set_height(300);
        video
    }

    pub fn get_peer(&self) -> JsValue {
        self.peer.as_ref().clone().unchecked_into()
    }

    #[wasm_bindgen(constructor)]
    pub fn new(dom_element: web_sys::Element) -> Streaming {
        let video1 = Streaming::create_muted_video(true);
        let video2 = Streaming::create_muted_video(false);
        let mut config = RtcConfiguration::new();
        let obj = js_sys::Object::new();
        let arr = js_sys::Array::new();
        set![obj => "urls", "stun:senges.ch:3478"];
        /*let obj2 = js_sys::Object::new();
        set![obj2 => "urls", "turn:senges.ch:3478"];
        arr.push(&obj2);*/

        arr.push(&obj);
        console::log_1(&arr);
        config.ice_servers(&arr);
        //let peer: Arc<RtcPeerConnection> = Arc::new(RtcPeerConnection::new_with_configuration(&config).unwrap());
        let peer: Arc<RtcPeerConnection> = Arc::new(RtcPeerConnection::new().unwrap());

        Streaming {
            dom_element,
            video1: Rc::new(video1),
            video2: Rc::new(video2),
            peer,
            on_ice_candidate: js_sys::Function::new_no_args(""),
        }
    }

    pub fn set_on_ice_candidate(&mut self, closure: js_sys::Function) {
        self.on_ice_candidate = closure;
    }

    pub fn add_ice_candidate(&mut self, candidate: RtcIceCandidate) {
        let _ = self.peer.as_ref().add_ice_candidate_with_opt_rtc_ice_candidate(Option::from(&candidate));
    }

    pub fn load_video(&self) -> js_sys::Promise {
        let devices = web_sys::window().unwrap().navigator().media_devices().unwrap();
        let mut constraints = MediaStreamConstraints::new();
        constraints.audio(&JsValue::TRUE);
        constraints.video(&JsValue::TRUE);
        let promise = devices.get_user_media_with_constraints(&constraints).unwrap();
        let video1 = Rc::clone(&self.video1);
        let video2 = Rc::clone(&self.video2);
        self.dom_element.append_child(&video1).unwrap();
        self.dom_element.append_child(&video2).unwrap();
        let peer = self.peer.clone();

        future_to_promise(async move {
            let js_stream: JsValue = js_await![promise];
            let stream: MediaStream = js_stream.unchecked_into();
            stream.get_tracks().iter().for_each(|track: JsValue| {
                peer.add_track_0(&track.unchecked_into(), &stream);
            });
            video1.set_src_object(Some(&stream));
            Ok(stream.unchecked_into())
        })
    }

    fn ice_candidate_cb(&self) -> Closure<dyn FnMut(JsValue)> {
        let cb = self.on_ice_candidate.clone();
        Closure::wrap(Box::new(move |event: JsValue| {
            match get![event => "candidate"].dyn_into::<RtcIceCandidate>() {
                Ok(candidate) => {
                    cb.call1(&JsValue::NULL, &candidate).unwrap();
                }
                Err(_e) => {}
            };
        }) as Box<dyn FnMut(JsValue)>)
    }

    fn track_cb(&self) -> Closure<dyn FnMut(JsValue)> {
        let video2 = Rc::clone(&self.video2);
        Closure::wrap(Box::new(move |event: JsValue| {
            log("In track");
            let video: &HtmlVideoElement = video2.as_ref();
            match video2.src_object() {
                Some(_video) => {
                    let streams: js_sys::Array = get![event => "streams"].unchecked_into();
                    let js_stream: JsValue = streams.get(0);
                    let stream: MediaStream = js_stream.unchecked_into();
                    video.set_src_object(Some(&stream));
                }
                None => {
                    let streams: js_sys::Array = get![event => "streams"].unchecked_into();
                    let js_stream: JsValue = streams.get(0);
                    let stream: MediaStream = js_stream.unchecked_into();
                    video.set_src_object(Some(&stream));
                }
            }
        }) as Box<dyn FnMut(JsValue)>)
    }

    pub fn accept_offer(&self, offer: RtcSessionDescriptionInit) -> ConnectionOffer {
        let peer = Arc::clone(&self.peer);

        let on_track = self.track_cb();

        let cb1 = self.ice_candidate_cb();
        peer.set_onicecandidate(cb1.as_ref().dyn_ref());

        peer.set_ontrack(on_track.as_ref().dyn_ref());

        let p = future_to_promise(async move {
            let set_remote_promise = peer.as_ref().set_remote_description(&offer);
            js_await![set_remote_promise];


            let answer_promise = peer.as_ref().create_answer();
            let js_answer: JsValue = js_await![answer_promise];
            let answer: RtcSessionDescriptionInit = js_answer.unchecked_into();
            js_await![peer.as_ref().set_local_description(&answer)];
            Ok(answer.unchecked_into())
        });

        let mut cb_ret = ConnectionOffer::new(p);
        cb_ret.add_cb(on_track);
        cb_ret.add_cb(cb1);
        cb_ret
    }

    pub fn accept_answer(&self, answer: RtcSessionDescriptionInit) -> ConnectionOffer {
        let peer = Arc::clone(&self.peer);

        let p = future_to_promise(async move {
            let set_remote_promise = peer.as_ref().set_remote_description(&answer);
            js_await![set_remote_promise];
            Ok(JsValue::TRUE)
        });

        let cb_ret = ConnectionOffer::new(p);
        cb_ret
    }

    pub fn create_offer(&self) -> ConnectionOffer {
        let peer = Arc::clone(&self.peer);

        let on_track = self.track_cb();

        let cb1 = self.ice_candidate_cb();

        peer.set_onicecandidate(cb1.as_ref().dyn_ref());

        peer.set_ontrack(on_track.as_ref().dyn_ref());

        let p = future_to_promise(async move {
            let mut options: RtcOfferOptions = RtcOfferOptions::new();
            options.offer_to_receive_audio(true);
            options.offer_to_receive_video(true);
            let promise = peer.create_offer_with_rtc_offer_options(&options);

            let js_offer: JsValue = js_await![promise];
            let offer: RtcSessionDescriptionInit = js_offer.unchecked_into();
            let set_local_promise = peer.as_ref().set_local_description(&offer);
            js_await![set_local_promise];
            Ok(offer.unchecked_into())
        });

        let mut cb_ret = ConnectionOffer::new(p);
        cb_ret.add_cb(on_track);
        cb_ret.add_cb(cb1);
        cb_ret
    }
}
