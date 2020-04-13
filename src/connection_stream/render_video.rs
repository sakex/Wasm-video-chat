use web_sys::*;
use std::rc::Rc;
use wasm_bindgen::prelude::{JsValue, Closure};
use wasm_bindgen::JsCast;
use std::cell::RefCell;


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
    videos: Rc<RefCell<Vec<VideoPos>>>,
    next_pos: (f64, f64),
}

impl VideoRenderer {
    pub fn new(canvas_rc: Rc<HtmlCanvasElement>) -> Result<VideoRenderer, JsValue> {
        let context = canvas_rc
            .get_context("2d")?
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()?;
        Ok(VideoRenderer {
            context_rc: Rc::new(context),
            videos: Rc::new(RefCell::new(Vec::new())),
            next_pos: (10.0, 10.0),
        })
    }

    pub fn add_video(&mut self, video_rc: Rc<HtmlVideoElement>) {
        let (x, y) = self.next_pos;
        if x < 980.0 {
            self.next_pos.0 += 310.0;
        } else {
            self.next_pos.0 = 10.0;
            self.next_pos.1 += 310.0;
        }
        let pos = VideoPos {
            video_rc,
            x,
            y,
        };
        self.videos.borrow_mut().push(pos);
    }

    pub fn remove_video(&mut self, video_rc: &Rc<HtmlVideoElement>) {
        let mut videos = self.videos.borrow_mut();
        let index = videos.iter().position(|video_pos| Rc::ptr_eq(&video_rc, &video_pos.video_rc)).unwrap();
        let vid = videos.remove(index);
        self.context_rc.clear_rect(vid.x, vid.y, 300.0, 300.0);
        for i in index..videos.len() {
            self.context_rc.clear_rect(videos[i].x, videos[i].y, 300.0, 300.0);
            if videos[i].x == 10.0 {
                videos[i].x = 930.0;
                videos[i].y -= 310.0;
            }
            else {
                videos[i].x -= 310.0;
            }
        }
        if self.next_pos.0 == 10.0 {
            self.next_pos.0 = 930.0;
            self.next_pos.1 -= 310.0;
        } else {
            self.next_pos.0 -= 310.0;
        }
    }

    pub fn start(&self) -> Result<DrawCb, JsValue> {
        let func = Rc::new(RefCell::new(None));
        let func_cp = func.clone();
        let videos = self.videos.clone();
        let context = self.context_rc.clone();
        *func_cp.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            for video_pos in videos.borrow().iter() {
                context.draw_image_with_html_video_element_and_dw_and_dh(
                    video_pos.video_rc.as_ref(), video_pos.x, video_pos.y, 300.0, 300.0).unwrap();
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
