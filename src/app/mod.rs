pub mod engine;
pub mod viewport;
pub mod world;

#[derive(Default)]
pub struct App {
    input: winit_input_helper::WinitInputHelper,

    engine: Option<engine::Engine>,
}

impl winit::application::ApplicationHandler<()> for App {
    fn resumed(&mut self, p_event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.engine.is_none() {
            let builder = engine::EngineBuilder::try_default().unwrap();
            let engine = builder
                .build(engine::EngineBuilderParameters {
                    event_loop: p_event_loop,
                })
                .unwrap();
            self.engine = Some(engine);
        }
    }

    fn window_event(
        &mut self,
        _: &winit::event_loop::ActiveEventLoop,
        _: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if self.input.process_window_event(&event) {
            if let Some(engine) = &mut self.engine {
                engine.render().unwrap();
            }
        }
    }

    fn device_event(
        &mut self,
        _: &winit::event_loop::ActiveEventLoop,
        _: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        // pass in events
        self.input.process_device_event(&event);
    }

    fn about_to_wait(&mut self, p_event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(engine) = &mut self.engine {
            engine.process(&self.input, p_event_loop);
        }
    }

    fn new_events(&mut self, _: &winit::event_loop::ActiveEventLoop, _: winit::event::StartCause) {
        self.input.step();
    }
}

impl App {
    pub fn run() -> anyhow::Result<()> {
        env_logger::init();
        let event_loop = winit::event_loop::EventLoop::new()?;
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        let mut app = App::default();
        event_loop.run_app(&mut app)?;
        Ok(())
    }
}
