use std::iter;

use anyhow::{Context, Result};
use pollster::FutureExt;
use wgpu::{
    Adapter, CommandEncoderDescriptor, CompositeAlphaMode, Device, DeviceDescriptor, Features,
    Instance, InstanceDescriptor, Limits, LoadOp, Operations, PresentMode, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, Surface,
    SurfaceConfiguration, TextureUsages, TextureViewDescriptor,
};
use winit::{
    event::{ElementState, Event, WindowEvent},
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
    let (event_loop, mut state) = State::new()?;

    event_loop.run(move |event, _, flow| {
        let result = match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::MouseInput {
                    state: ElementState::Pressed,
                    ..
                } => {
                    state.cursor_visible = !state.cursor_visible;
                    state.window.set_cursor_visible(state.cursor_visible);
                    state.window.request_redraw();

                    dbg!(state.cursor_visible);
                    Ok(())
                }
                WindowEvent::CursorMoved { .. } => {
                    state.cursor_visible = true;
                    state.window.set_cursor_visible(true);
                    state.window.request_redraw();

                    dbg!(state.cursor_visible);
                    Ok(())
                }
                WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
                    state.reconfigure_surface();
                    Ok(())
                }
                WindowEvent::CloseRequested => {
                    *flow = ControlFlow::Exit;
                    Ok(())
                }
                _ => Ok(()),
            },
            Event::RedrawRequested(_) => state.draw().context("Could not draw next frame"),
            _ => Ok(()),
        };

        if let Err(err) = result {
            eprintln!("{err}");
            *flow = ControlFlow::ExitWithCode(1);
        }
    })
}

struct State {
    adapter: Adapter,
    device: Device,
    queue: Queue,
    surface: Surface,

    window: Window,

    cursor_visible: bool,
}

impl State {
    fn new() -> Result<(EventLoop<()>, Self)> {
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

        configure_surface(&surface, &device, &adapter, &window);

        Ok((
            event_loop,
            State {
                adapter,
                device,
                queue,
                surface,
                window,
                cursor_visible: false,
            },
        ))
    }

    fn draw(&mut self) -> Result<()> {
        // very crude handling, the swapchain could be destroyed easily, but eh
        let next_frame = self
            .surface
            .get_current_texture()
            .context("Could not ask surface for the next texture")?;

        let preferred_format = self.surface.get_capabilities(&self.adapter).formats[0];
        let next_frame_view = next_frame.texture.create_view(&TextureViewDescriptor {
            format: Some(preferred_format),
            ..TextureViewDescriptor::default()
        });

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());

        let background_color = match self.cursor_visible {
            false => wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.4,
                a: 1.0,
            },
            true => wgpu::Color {
                r: 0.1,
                g: 1.0,
                b: 0.2,
                a: 1.0,
            },
        };
        let render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &next_frame_view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(background_color),
                    store: true,
                },
            })],
            ..RenderPassDescriptor::default()
        });
        drop(render_pass);

        self.queue.submit(iter::once(encoder.finish()));
        next_frame.present();

        Ok(())
    }

    fn reconfigure_surface(&self) {
        configure_surface(&self.surface, &self.device, &self.adapter, &self.window);
    }
}

fn configure_surface(surface: &Surface, device: &Device, adapter: &Adapter, window: &Window) {
    let preferred_format = surface.get_capabilities(adapter).formats[0];
    let window_size = window.inner_size();
    surface.configure(
        device,
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
}
