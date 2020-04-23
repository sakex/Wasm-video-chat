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
    dims: Rc<RefCell<(f64, f64)>>,
    managed: bool,
    width_height: Rc<RefCell<(f64, f64)>>,
    _on_resize: Closure<dyn FnMut()>,
}

impl VideoRenderer {
    #[inline]
    fn on_resize(canvas_rc: Rc<HtmlCanvasElement>, width_height: Rc<RefCell<(f64, f64)>>) -> Closure<dyn FnMut()> {
        let mut wh = *width_height.borrow_mut();
        let cv2 = canvas_rc.clone();
        let closure = Closure::wrap(Box::new(move || {
            let new_width = canvas_rc.offset_width();
            let new_height = canvas_rc.offset_height();
            wh.0 = new_width as f64;
            wh.1 = new_height as f64;
        }) as Box<dyn FnMut()>);
        cv2.set_onresize(closure.as_ref().dyn_ref());
        closure
    }

    pub fn new(canvas_rc: Rc<HtmlCanvasElement>, width: i32, height: i32) -> Result<VideoRenderer, JsValue> {
        let context = canvas_rc
            .get_context("2d")?
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()?;
        let f_width = width as f64;
        let f_height = height as f64;
        let width_height = Rc::new(RefCell::new((f_width.clone(), f_height.clone())));
        let _on_resize = VideoRenderer::on_resize(canvas_rc, width_height.clone());
        let renderer = VideoRenderer {
            context_rc: Rc::new(context),
            videos: Rc::new(RefCell::new(HashMap::new())),
            next_pos: (10.0, 10.0),
            dims: Rc::new(RefCell::new((f_width, f_height))),
            managed: true,
            width_height,
            _on_resize,
        };

        Ok(renderer)
    }

    #[inline]
    pub fn clear_all(&self) {
        let (width, height) = *self.width_height.borrow();
        self.context_rc.clear_rect(0., 0., width, height);
    }

    #[inline]
    pub fn set_dims(&mut self, x: f64, y: f64) {
        self.dims.borrow_mut().0 = x;
        self.dims.borrow_mut().1 = y;
        // console::log_1(&format!("{}, {}", f_width, f_height).into());
    }

    #[inline]
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

    #[inline]
    pub fn not_managed(&mut self) {
        self.managed = false;
    }


    fn update_count(&mut self, count: f64) {
        let width_height = *self.width_height.borrow();
        let possible_width = (width_height.0 - 10. * count) / count;
        if possible_width > 210. {
            self.set_dims(possible_width, possible_width);
            let mut current_x: f64 = 0.;
            let mut current_y: f64 = 0.;
            for (_, video_pos) in &mut *self.videos.as_ref().borrow_mut() {
                video_pos.x = current_x;
                current_x += possible_width + 10.;
                if current_x + self.dims.borrow().0 >= width_height.0 {
                    current_x = 0.;
                    current_y += width_height.1 + 10.;
                    video_pos.y = possible_width + 10.;
                }
            }
            self.next_pos = (current_x, current_y);
            self.clear_all();
        }
    }

    fn video_pos_managed(&mut self, video_rc: Rc<HtmlVideoElement>) -> VideoPos {
        let count = self.videos.as_ref().borrow().len() as f64 + 1.;
        self.update_count(count);
        let dims = *self.dims.borrow();
        let (x, y) = self.next_pos;
        let (width, _height) = *self.width_height.borrow();
        if x + dims.0 < width {
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

    #[inline]
    pub fn add_video(&mut self, id: String, video_rc: Rc<HtmlVideoElement>) {
        let pos = match self.managed {
            true => self.video_pos_managed(video_rc),
            false => self.video_pos_not_managed(video_rc)
        };
        self.videos.borrow_mut().insert(id, pos);
    }

    pub fn remove_video(&mut self, id: &String) {
        let count: f64 = {
            let videos_rc = self.videos.clone();
            let mut videos = videos_rc.borrow_mut();
            let vid = videos.remove(id).unwrap();
            let (width, height) = *self.dims.borrow();
            self.context_rc.clear_rect(vid.x, vid.y, width, height);
            videos.len() as f64
        };
        if !self.managed {
            return;
        }
        self.update_count(count);
    }

    pub fn start(&self) -> Result<DrawCb, JsValue> {
        let func = Rc::new(RefCell::new(None));
        let func_cp = func.clone();
        let videos = self.videos.clone();
        let context = self.context_rc.clone();
        let dims_rc = self.dims.clone();
        *func_cp.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            let dims = dims_rc.borrow();
            for (_, video_pos) in videos.borrow().iter() {
                context.draw_image_with_html_video_element_and_dw_and_dh(
                    video_pos.video_rc.as_ref(), video_pos.x, video_pos.y, dims.0, dims.1).unwrap();
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
