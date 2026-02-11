use laufen::adapter::renderer::*;

pub mod quad;

pub struct Viewport {
    rendering_server: rendering_server::RenderingServer,
}

impl super::Update for Viewport {
    fn update(&mut self, _delta: std::time::Duration) {}
}

impl Viewport {
    pub fn new(p_event_loop: &winit::event_loop::ActiveEventLoop) -> anyhow::Result<Self> {
        let window_attributes = winit::window::Window::default_attributes();
        let window = std::sync::Arc::new(p_event_loop.create_window(window_attributes).unwrap());
        let builder = rendering_server::RenderingServerBuilder::default();
        let rendering_server = pollster::block_on(
            builder.build(rendering_server::RenderingServerBuilderParameters { window }),
        )?;

        Ok(Self { rendering_server })
    }

    pub fn resize(&mut self, p_width: u32, p_height: u32) {
        self.rendering_server.resize(p_width, p_height);
    }
}
