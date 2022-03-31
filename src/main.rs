mod app;

use winit::{event_loop::EventLoop, window::WindowBuilder};

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Mandelbddap")
        .build(&event_loop)
        .unwrap();

    #[cfg(not(target_arch = "wasm32"))]
    {
        pollster::block_on(app::run(event_loop, window));
    }
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::closure::Closure;
        use wasm_bindgen::JsCast;
        use winit::platform::web::WindowExtWebSys;

        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Debug).expect("error initializing logger");

        // stick the canvas onto the page
        let body = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .body()
            .unwrap();
        body.append_child(&window.canvas()).unwrap();

        window.canvas().style().set_css_text("");

        // keep canvas pixel size up to date with actual size
        let canvas = window.canvas();
        let on_resize: Box<dyn FnMut()> = Box::new(move || {
            canvas.set_width(canvas.client_width() as u32);
            canvas.set_height(canvas.client_height() as u32);
        });
        let on_resize = Closure::wrap(on_resize);
        body.set_onresize(Some(on_resize.as_ref().unchecked_ref()));
        on_resize.forget();

        wasm_bindgen_futures::spawn_local(app::run(event_loop, window));
    }
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen(start))]
pub fn wasm_main() {
    main();
}
