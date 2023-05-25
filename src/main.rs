use anyhow::{Context, Result};
use pollster::FutureExt;
use wgpu::{
    CompositeAlphaMode, Device, DeviceDescriptor, Features, Instance, InstanceDescriptor, Limits,
    MultisampleState, PresentMode, PrimitiveState, Queue, RenderPipeline, RenderPipelineDescriptor,
    RequestAdapterOptions, ShaderModuleDescriptor, ShaderSource, Surface, SurfaceConfiguration,
    TextureUsages, VertexState,
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

/// The never type ([`!`]) but not since it's not stable yet.
enum Never {}

fn run() -> Result<Never> {
    let state = State::new()?;

    state.event_loop.run(|event, _, flow| {
        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            *flow = ControlFlow::Exit;
        }
    })
}

struct State {
    device: Device,
    queue: Queue,
    surface: Surface,
    pipeline: RenderPipeline,

    event_loop: EventLoop<()>,
    window: Window,
}

impl State {
    fn new() -> Result<Self> {
        let event_loop = EventLoop::new();
        let window = Window::new(&event_loop)?;

        let instance = Instance::new(InstanceDescriptor::default());
        // SAFETY: window was just created and is dropped after the surface due to State's drop
        // order
        let surface = unsafe { instance.create_surface(&window) }?;

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .block_on()
            .context("Found no appropiate adapter")?;

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    features: Features::empty(),
                    limits: Limits::downlevel_webgl2_defaults(),
                },
                None,
            )
            .block_on()
            .context("Found no appropiate device")?;

        let preferred_format = surface.get_capabilities(&adapter).formats[0];
        let window_size = window.inner_size();
        surface.configure(
            &device,
            &SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT,
                format: preferred_format,
                width: window_size.width,
                height: window_size.height,
                present_mode: PresentMode::Fifo,
                alpha_mode: CompositeAlphaMode::Auto,
                view_formats: Vec::new(),
            },
        );

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(
                // all we're doing is clearing, no real shader needed
                "@vertex fn placeholder() -> @builtin(position) vec4<f32> { return vec4(0.0, 0.0, 0.0, 0.0 ); }".into()
            ),
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: None,
            vertex: VertexState {
                module: &shader,
                entry_point: "placeholder",
                buffers: &[],
            },
            fragment: None,
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
        });

        Ok(State {
            device,
            queue,
            surface,
            pipeline,
            event_loop,
            window,
        })
    }
}
