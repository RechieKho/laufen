pub mod bridge;
pub mod net;
pub mod renderer;
pub mod script;

use renderer::rendering_server;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    window::Window,
};

#[allow(unused)]
pub use net::start_sample_net;

#[allow(unused)]
pub use script::run_sample_script;

#[allow(unused)]
pub struct UserEvent {
    pub code: u8,
    pub content: String,
}

pub struct App {
    rendering_server: Option<rendering_server::RenderingServer>,
    rendering_context: Option<renderer::SampleRenderingContext>,
}

impl App {
    pub fn new() -> Self {
        Self {
            rendering_server: None,
            rendering_context: None,
        }
    }
}

impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes();
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        let builder = rendering_server::RenderingServerBuilder::default();
        self.rendering_server = Some(
            pollster::block_on(
                builder.build(rendering_server::RenderingServerBuilderParameters { window }),
            )
            .unwrap(),
        );
        if let Some(server) = &self.rendering_server {
            self.rendering_context = Some(renderer::SampleRenderingContext::new(server).unwrap());
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(server) = &mut self.rendering_server {
                    server.resize(size.width, size.height);
                }
            }
            WindowEvent::RedrawRequested => {
                if let (Some(context), Some(server)) =
                    (&mut self.rendering_context, &mut self.rendering_server)
                {
                    match context.render(server) {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            server.resize_to_window();
                        }
                        Err(e) => {
                            log::error!("Unable to render {}", e);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

#[allow(unused)]
pub fn run_app() -> anyhow::Result<()> {
    env_logger::init();

    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = App::new();
    event_loop.run_app(&mut app)?;

    Ok(())
}
