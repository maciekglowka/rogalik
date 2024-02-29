use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event_loop::EventLoop,
    platform::web::{WindowExtWebSys, WindowBuilderExtWebSys},
    window::{Window, WindowBuilder}
};
use wasm_bindgen::JsCast;

pub fn get_window(event_loop: &EventLoop<()>) -> Window {
    let canvas = web_sys::window()
        .and_then(|win| {
            win.document()
        })
        .and_then(|doc| {
            let element = doc.get_element_by_id("rogalik-canvas")?;
            log::info!("Found #rogalik-canvas");
            Some(element.dyn_into::<web_sys::HtmlCanvasElement>()
                .map_err(|_| ())
                .expect("Html element is not a canvas!"))
        })
        .expect("Can't find canvas!");

    let mut window_builder = WindowBuilder::new();
    let window = window_builder
        .with_canvas(Some(canvas))
        .build(event_loop)
        .expect("Can't create window!");
    window
}

pub fn configure_handlers() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Warning).expect("Can't init the logger!");
}
