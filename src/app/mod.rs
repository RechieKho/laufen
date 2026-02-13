use viewport::quad;

pub mod viewport;

pub trait Update {
    fn update(&mut self, p_delta: std::time::Duration);
}

pub struct App {
    viewport: Option<viewport::Viewport>,
    last_timestamp: std::time::Instant,
}

impl Default for App {
    fn default() -> Self {
        Self {
            viewport: None,
            last_timestamp: std::time::Instant::now(),
        }
    }
}

impl winit::application::ApplicationHandler<()> for App {
    fn resumed(&mut self, p_event_loop: &winit::event_loop::ActiveEventLoop) {
        self.viewport = Some(viewport::Viewport::new(p_event_loop).unwrap());
    }

    fn window_event(
        &mut self,
        p_event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        p_event: winit::event::WindowEvent,
    ) {
        let current_timestamp = std::time::Instant::now();
        let _delta = current_timestamp.duration_since(self.last_timestamp);

        match p_event {
            winit::event::WindowEvent::CloseRequested => p_event_loop.exit(),
            winit::event::WindowEvent::Resized(size) => {
                if let Some(viewport) = &mut self.viewport {
                    viewport.resize(size.width, size.height);
                }
            }
            winit::event::WindowEvent::RedrawRequested => {
                if let Some(viewport) = &mut self.viewport {
                    viewport
                        .render(viewport::ViewportRenderParameters {
                            quad_instances: &[quad::Instance::from(
                                quad::QuadRenderPipelineContext::QUAD_BACKWARD_MATRIX,
                            )],
                        })
                        .unwrap();
                }
            }
            _ => {}
        }

        self.last_timestamp = current_timestamp;
    }
}

impl App {
    pub fn run() -> anyhow::Result<()> {
        env_logger::init();
        let event_loop = winit::event_loop::EventLoop::new()?;
        let mut app = App::default();
        event_loop.run_app(&mut app)?;
        Ok(())
    }
}
