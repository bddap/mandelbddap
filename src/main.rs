mod app;

use winit::{
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let (event_loop, window) = ev_window();
        pollster::block_on(app::run(event_loop, window));
    }
    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;

        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Debug).expect("error initializing logger");
        let (event_loop, window) = ev_window();

        // stick the canvas onto the page
        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .body()
            .unwrap()
            .append_child(&window.canvas())
            .unwrap();

        wasm_bindgen_futures::spawn_local(app::run(event_loop, window));
    }
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen(start))]
pub fn wasm_main() {
    main();
}

fn ev_window() -> (EventLoop<()>, Window) {
    let ev = EventLoop::new();
    let win = WindowBuilder::new()
        .with_title("Mandelbddap")
        .build(&ev)
        .unwrap();
    (ev, win)
}
