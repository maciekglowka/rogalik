use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event_loop::EventLoop,
    platform::web::WindowExtWebSys,
    window::{Window, WindowBuilder}
};

pub fn get_window(event_loop: &EventLoop<()>) -> Window {
    let mut window_builder = WindowBuilder::new();
    let window = window_builder.build(event_loop)
        .expect("Can't create window!");

    log::info!("Got window");

    web_sys::window()
        .and_then(|win| {
            let width = win.inner_width().unwrap().as_f64().unwrap() as u32;
            let height = win.inner_height().unwrap().as_f64().unwrap() as u32;
            log::info!("Canvas size: {}, {}", width, height);
            let window_size = LogicalSize::new(width, height);
            window.set_inner_size(window_size);
            win.document()
        })
        .and_then(|doc| {
            let body = doc.get_elements_by_tag_name("body").get_with_index(0)?;
            let canvas = window.canvas();
            let canvas_element = web_sys::Element::from(canvas);
            body.append_child(&canvas_element).ok()?;
            Some(())
        })
        .expect("Can't append canvas!");
    window
}

pub fn configure_handlers() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Debug).expect("Can't init the logger!");
}
