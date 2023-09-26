use winit::{
    platform::web::WindowExtWebSys,
    window::Window,
};

pub fn set_canvas(window: &Window) {
    use winit::dpi::PhysicalSize;
        window.set_inner_size(PhysicalSize::new(800, 600));

        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let body = doc.get_elements_by_tag_name("body").get_with_index(0)?;
                let canvas = web_sys::Element::from(window.canvas());
                body.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Can't append canvas!");
}

pub fn configure_handlers() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Warn).expect("Can't init logger!");
}