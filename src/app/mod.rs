use crate::adapter::net;

pub mod arch;
pub mod biosphere;
pub mod engine;
pub mod geosphere;
pub mod viewport;

#[derive(Default)]
pub struct App {
    input: winit_input_helper::WinitInputHelper,

    engine: Option<engine::Engine>,
}

const SAMPLE_CLIENT_ADDRESS: std::net::SocketAddr = std::net::SocketAddr::new(
    std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
    0,
);
const SAMPLE_SERVER_ADDRESS: std::net::SocketAddr = std::net::SocketAddr::new(
    std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
    5000,
);

impl winit::application::ApplicationHandler<()> for App {
    fn resumed(&mut self, p_event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.engine.is_none() {
            let builder = engine::UnsecureEngineBuilder::try_with_sample().unwrap();
            let engine = builder
                .build(engine::UnsecureEngineBuilderParameters {
                    event_loop: p_event_loop,
                    provider_builder_parameters: arch::provider::ProviderBuilderParameters {
                        server_context_builder_parameters:
                            net::server::ServerContextBuilderParameters {
                                address: SAMPLE_SERVER_ADDRESS,
                            },
                    },
                    consumer_builder_parameters:
                        arch::consumer::UnsecureConsumerBuilderParameters {
                            client_context_builder_parameters:
                                net::client::UnsecureClientContextBuilderParameters {
                                    server_address: SAMPLE_SERVER_ADDRESS,
                                    client_address: SAMPLE_CLIENT_ADDRESS,
                                    client_id: 0,
                                },
                        },
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
