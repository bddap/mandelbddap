use std::borrow::Cow;
use wgpu::{
    Adapter, Device, Instance, PipelineLayout, Queue, RenderPipeline, ShaderModule, Surface,
    SurfaceConfiguration,
};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

struct App {
    config: SurfaceConfiguration,
    surface: Surface,
    window: Window,
    device: Device,
    queue: Queue,
    render_pipeline: RenderPipeline,

    // things that aren't used during the event loop, but need to stay in scope for
    // the lifetime of the app
    _instance: Instance,
    _adapter: Adapter,
    _shader: ShaderModule,
    _pipeline_layout: PipelineLayout,
}

impl App {
    async fn new(window: Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                // Request an adapter which can render to our surface
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");

        // Create the logical device and command queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                    limits: wgpu::Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                },
                None,
            )
            .await
            .expect("Failed to create device");

        // Load the shaders from disk
        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(SHADER)),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let swapchain_format = surface.get_preferred_format(&adapter).unwrap();

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[swapchain_format.into()],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        surface.configure(&device, &config);

        Self {
            config,
            surface,
            window,
            device,
            queue,
            render_pipeline,
            _instance: instance,
            _adapter: adapter,
            _shader: shader,
            _pipeline_layout: pipeline_layout,
        }
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        // Reconfigure the surface with the new size
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
        // On macos the window needs to be redrawn manually after resizing
        self.window.request_redraw();
    }

    fn cursor_moved(&mut self, new_position: PhysicalPosition<f64>) {
        log::debug!("{:?}", new_position);
    }

    fn redraw(&mut self) {
        let frame = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            rpass.set_pipeline(&self.render_pipeline);
            rpass.draw(0..3, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    fn event(&mut self, event: Event<()>) -> ControlFlow {
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => self.resize(size),
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => self.cursor_moved(position),
            Event::RedrawRequested(_) => self.redraw(),
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => return ControlFlow::Exit,
            _ => {}
        }
        ControlFlow::Wait
    }

    fn runloop(mut self, event_loop: EventLoop<()>) {
        event_loop.run(move |event, _, control_flow| {
            *control_flow = self.event(event);
        })
    }
}

pub async fn run(event_loop: EventLoop<()>, window: Window) {
    let app = App::new(window).await;
    app.runloop(event_loop);
}

const SHADER: &str = "
struct Input {
  cursor : vec2<f32>;
};

[[group(0), binding(0)]] var<uniform> input : Input;

struct VertexOutput {
    [[location(0)]] tex_coord: vec2<f32>;
    [[builtin(position)]] position: vec4<f32>;
};

[[stage(vertex)]]
fn vs_main([[builtin(vertex_index)]] in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(i32(in_vertex_index) - 1);
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1);
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.tex_coord = vec2<f32>(x, y);
    return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    // return vec4<f32>(1.0, 0.0, 0.0, 1.0);
    return vec4<f32>((in.tex_coord + vec2<f32>(1.0, 1.0)) / 2.0, 0.0, 1.0);
}
";
