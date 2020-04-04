use wasm_bindgen::prelude::*;
use web_sys::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::future_to_promise;
use std::rc::Rc;
use wasm_bindgen::__rt::core::ptr::null;
use crate::js_extend::Gettable;


#[wasm_bindgen]
pub struct Streaming {
    dom_element: web_sys::Element,
    video1: Rc<web_sys::HtmlVideoElement>,
    video2: Rc<web_sys::HtmlVideoElement>,
    peer1: Rc<RtcPeerConnection>,
    peer2: Rc<RtcPeerConnection>,
}

#[wasm_bindgen]
impl Streaming {
    fn create_muted_video() -> web_sys::HtmlVideoElement {
        let document = web_sys::window().unwrap().document().unwrap();
        let video = document.create_element("video").unwrap().unchecked_into::<web_sys::HtmlVideoElement>();
        video.set_autoplay(true);
        video.set_muted(true);
        video.set_width(300);
        video.set_height(300);
        video
    }

    #[wasm_bindgen(constructor)]
    pub fn new(dom_element: web_sys::Element) -> Streaming {
        let video1 = Streaming::create_muted_video();
        let video2 = Streaming::create_muted_video();
        let peer1: Rc<RtcPeerConnection> = Rc::new(RtcPeerConnection::new().unwrap());
        let peer2: Rc<RtcPeerConnection> = Rc::new(RtcPeerConnection::new().unwrap());
        Streaming { dom_element, video1: Rc::new(video1), video2: Rc::new(video2), peer1, peer2 }
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

        future_to_promise(async move {
            let js_stream: JsValue = wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
            let stream: MediaStream = js_stream.unchecked_into();
            video1.set_src_object(Some(&stream));
            Ok(JsValue::TRUE)
        })
    }


    fn ice_candidate_cb(peer: Rc<RtcPeerConnection>) -> Closure<dyn FnMut(JsValue)> {
        Closure::wrap(Box::new(move |event: JsValue| {
            let candidate: RtcIceCandidate = event.get("candidate").dyn_into().unwrap();
            peer.as_ref().add_ice_candidate_with_opt_rtc_ice_candidate(Some(&candidate));
        }) as Box<dyn FnMut(JsValue)>)
    }

    fn track_cb(&self) -> Closure<dyn FnMut(JsValue)> {
        let video2 = Rc::clone(&self.video2);
        Closure::wrap(Box::new(move |event: JsValue| {
            let streams: js_sys::Array = event.get("streams").unchecked_into();
            let js_stream: JsValue = streams.get(0);
            let stream: MediaStream = js_stream.unchecked_into();
            let video: &HtmlVideoElement = video2.as_ref();
            video.set_src_object(Some(&stream));
        }) as Box<dyn FnMut(JsValue)>)
    }

    pub fn call(&self) {
        let video1: &HtmlVideoElement = self.video1.as_ref();
        let video_tracks = video1.video_tracks();
        let audio_tracks = video1.audio_tracks();

        let p1 = Rc::clone(&self.peer1);
        let p2 = Rc::clone(&self.peer2);

        self.peer1.set_onicecandidate(Streaming::ice_candidate_cb(p1).as_ref().dyn_ref());
        self.peer2.set_onicecandidate(Streaming::ice_candidate_cb(p2).as_ref().dyn_ref());
        self.peer2.set_ontrack(self.track_cb().as_ref().dyn_ref());
    }
}
