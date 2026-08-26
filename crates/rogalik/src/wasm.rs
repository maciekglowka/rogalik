use wasm_bindgen::JsCast;

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

pub fn configure_handlers() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}
