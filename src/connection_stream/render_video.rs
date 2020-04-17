use web_sys::*;
use std::rc::Rc;
use wasm_bindgen::prelude::{JsValue, Closure};
use wasm_bindgen::JsCast;
use std::cell::RefCell;
use wasm_bindgen::__rt::std::collections::HashMap;


pub fn create_video(muted: bool) -> Result<Rc<HtmlVideoElement>, JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let video = document.create_element("video").unwrap().unchecked_into::<HtmlVideoElement>();
    video.set_autoplay(true);
    video.set_attribute("playsinline", "true").unwrap();
    video.set_muted(muted);
    video.style().set_property("display", "none")?;
    Ok(Rc::new(video))
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window().unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref()).unwrap();
}

type DrawCb = Rc<RefCell<Option<Closure<dyn FnMut()>>>>;

struct VideoPos {
    video_rc: Rc<HtmlVideoElement>,
    x: f64,
    y: f64,
}

pub struct VideoRenderer {
    context_rc: Rc<web_sys::CanvasRenderingContext2d>,
    videos: Rc<RefCell<HashMap<String, VideoPos>>>,
    next_pos: (f64, f64),
    dims: RefCell<(f64, f64)>,
    managed: bool,
}

impl VideoRenderer {
    pub fn new(canvas_rc: Rc<HtmlCanvasElement>) -> Result<VideoRenderer, JsValue> {
        let context = canvas_rc
            .get_context("2d")?
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()?;
        Ok(VideoRenderer {
            context_rc: Rc::new(context),
            videos: Rc::new(RefCell::new(HashMap::new())),
            next_pos: (10.0, 10.0),
            dims: RefCell::new((200., 200.)),
            managed: true,
        })
    }

    pub fn set_dims(&mut self, new_dims: (f64, f64)) {
        let mut dims = self.dims.borrow_mut();
        dims.0 = new_dims.0;
        dims.1 = new_dims.1;
    }

    pub fn set_video_pos(&mut self, id: String, x: f64, y: f64) -> Result<JsValue, JsValue> {
        match self.videos.borrow_mut().get_mut(&id) {
            Some(video) => {
                video.x = x;
                video.y = y;
                Ok(JsValue::TRUE)
            }
            None => Err(JsValue::FALSE)
        }
    }

    pub fn not_managed(&mut self) {
        self.managed = false;
    }

    fn video_pos_managed(&mut self, video_rc: Rc<HtmlVideoElement>) -> VideoPos {
        let (x, y) = self.next_pos;
        let dims = *self.dims.borrow();
        if x < 980.0 {
            self.next_pos.0 += dims.0 + 1.0;
        } else {
            self.next_pos.0 = 10.0;
            self.next_pos.1 += dims.1 + 10.0;
        }
        VideoPos {
            video_rc,
            x,
            y,
        }
    }

    fn video_pos_not_managed(&self, video_rc: Rc<HtmlVideoElement>) -> VideoPos {
        VideoPos {
            video_rc,
            x: -40000f64,
            y: -40000f64,
        }
    }

    pub fn add_video(&mut self, id: String, video_rc: Rc<HtmlVideoElement>) {
        let pos = match self.managed {
            true => self.video_pos_managed(video_rc),
            false => self.video_pos_not_managed(video_rc)
        };
        self.videos.borrow_mut().insert(id, pos);
    }

    pub fn remove_video(&mut self, id: &String) {
        let mut videos = self.videos.borrow_mut();
        let vid = videos.remove(id).unwrap();
        let dims = *self.dims.borrow();
        self.context_rc.clear_rect(vid.x, vid.y, dims.0, dims.1);
        if !self.managed {
            return;
        }
        for (_id, video) in &mut *videos {
            self.context_rc.clear_rect(video.x, video.y, dims.0, dims.1);
            if video.x == 10.0 {
                video.x = 930.0;
                video.y -= dims.1 + 10.;
            } else {
                video.x -= dims.0 + 10.;
            }
        }
        if self.next_pos.0 == 10.0 {
            self.next_pos.0 = 930.0;
            self.next_pos.1 -= dims.1 + 10.;
        } else {
            self.next_pos.0 -= 310.0;
        }
    }

    pub fn start(&self) -> Result<DrawCb, JsValue> {
        let func = Rc::new(RefCell::new(None));
        let func_cp = func.clone();
        let videos = self.videos.clone();
        let context = self.context_rc.clone();
        let dims = self.dims.clone();
        *func_cp.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            for (_, video_pos) in videos.borrow().iter() {
                context.draw_image_with_html_video_element_and_dw_and_dh(
                    video_pos.video_rc.as_ref(), video_pos.x, video_pos.y, dims.borrow().0, dims.borrow().1).unwrap();
            }
            request_animation_frame(func.borrow().as_ref().unwrap());
        }) as Box<dyn FnMut()>));

        request_animation_frame(func_cp.borrow().as_ref().unwrap());
        Ok(func_cp)
    }
}

/*pub fn get_canvas_stream(canvas: Rc<HtmlCanvasElement>, v: f64) -> Option<MediaStream> {
    let obj = canvas.as_ref().clone().unchecked_into::<JsValue>();
    let capture_stream = get![obj => "captureStream"].unchecked_into::<js_sys::Function>();
    let stream: JsValue = capture_stream.call1(&obj, &JsValue::from_f64(v)).unwrap();
    console::log_1(&stream);
    match stream.dyn_into() {
        Ok(s) => Some(s),
        Err(_e) => None
    }
}*/
