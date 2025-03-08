use wasm_bindgen::JsCast;
use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event_loop::EventLoop,
};

pub fn get_canvas() -> web_sys::HtmlCanvasElement {
    web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| {
            let element = doc.get_element_by_id("rogalik-canvas")?;
            log::info!("Found #rogalik-canvas");
            Some(
                element
                    .dyn_into::<web_sys::HtmlCanvasElement>()
                    .map_err(|_| ())
                    .expect("Html element is not a canvas!"),
            )
        })
        .expect("Can't find canvas!")
}

pub fn canvas_size() -> (u32, u32) {
    let canvas = get_canvas();
    (canvas.client_width() as u32, canvas.client_height() as u32)
}

pub fn configure_handlers() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    // console_log::init_with_level(log::Level::Warn).expect("Can't init the logger!");
}
