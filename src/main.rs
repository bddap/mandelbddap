use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

fn app(event_loop: EventLoop<()>, window: Window) {
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        log::debug!("{:?}", &event);

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => (),
        }
    });
}

fn main() {
    let (event_loop, window) = ev_window();
    app(event_loop, window);
}

fn ev_window() -> (EventLoop<()>, Window) {
    let ev = EventLoop::new();
    let win = WindowBuilder::new()
        .with_title("Mandelbddap")
        .build(&ev)
        .unwrap();
    (ev, win)
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    #[wasm_bindgen::prelude::wasm_bindgen(start)]
    pub fn run() {
        use winit::platform::web::WindowExtWebSys;

        console_log::init_with_level(log::Level::Debug).expect("error initializing logger");

        let (event_loop, window) = super::ev_window();

        // stick the canvas onto the page
        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .body()
            .unwrap()
            .append_child(&window.canvas())
            .unwrap();

        super::app(event_loop, window);
    }
}
