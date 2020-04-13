use web_sys::*;
use std::rc::Rc;
use wasm_bindgen::prelude::{JsValue, Closure};
use wasm_bindgen::JsCast;
use std::cell::RefCell;


pub fn create_video(muted: bool) -> Result<(Rc<HtmlVideoElement>, Rc<HtmlCanvasElement>), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let video = document.create_element("video").unwrap().unchecked_into::<HtmlVideoElement>();
    video.set_autoplay(true);
    video.set_attribute("playsinline", "true").unwrap();
    video.set_muted(muted);
    video.style().set_property("display", "none")?;
    let canvas = document.create_element("canvas").unwrap().unchecked_into::<HtmlCanvasElement>();
    canvas.set_width(300);
    canvas.set_height(300);
    Ok((Rc::new(video), Rc::new(canvas)))
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window().unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref()).unwrap();
}

type DrawCb = Rc<RefCell<Option<Closure<dyn FnMut()>>>>;

pub fn draw_video(video_rc: Rc<HtmlVideoElement>, canvas_rc: Rc<HtmlCanvasElement>) -> Result<DrawCb, JsValue> {
    let video = video_rc.clone();
    let context = canvas_rc
        .get_context("2d")?
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()?;
    let func = Rc::new(RefCell::new(None));
    let func_cp = func.clone();
    *func_cp.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        context.draw_image_with_html_video_element_and_dw_and_dh(video.as_ref(), 0.0, 0.0, 300.0, 300.0).unwrap();
        request_animation_frame(func.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(func_cp.borrow().as_ref().unwrap());
    Ok(func_cp)
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
