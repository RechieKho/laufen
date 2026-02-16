use viewport::quad;

pub mod viewport;
pub mod world;

#[derive(Default)]
pub struct App {
    viewport: Option<viewport::Viewport>,
    input: winit_input_helper::WinitInputHelper,
}

impl winit::application::ApplicationHandler<()> for App {
    fn resumed(&mut self, p_event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.viewport.is_none() {
            self.viewport = Some(viewport::Viewport::new(p_event_loop).unwrap());
        }
    }

    fn window_event(
        &mut self,
        _: &winit::event_loop::ActiveEventLoop,
        _: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if self.input.process_window_event(&event) {
            if let Some(viewport) = &mut self.viewport {
                viewport
                    .render(viewport::ViewportRenderParameters {
                        quad_instances: &[quad::QuadInstance::from(
                            quad::QuadRenderPipelineContext::QUAD_BACKWARD_MATRIX,
                        )],
                    })
                    .unwrap();
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
        if self.input.close_requested() {
            p_event_loop.exit();
            return;
        }

        if let Some((width, height)) = self.input.resolution() {
            if let Some(viewport) = &mut self.viewport {
                viewport.resize(width, height);
            }
        }

        if let Some(viewport) = &mut self.viewport {
            let camera_properties = viewport.camera_properties();
            const LINEAR_SPEED: f32 = 0.01;
            const ANGULAR_SPEED: f32 = std::f32::consts::PI / 100.0;

            if self.input.key_held(winit::keyboard::KeyCode::KeyW) {
                viewport.set_camera_properties(viewport::PartialViewportCameraProperties {
                    origin: Some(
                        camera_properties.origin + camera_properties.direction * LINEAR_SPEED,
                    ),
                    ..Default::default()
                });
            } else if self.input.key_held(winit::keyboard::KeyCode::KeyS) {
                viewport.set_camera_properties(viewport::PartialViewportCameraProperties {
                    origin: Some(
                        camera_properties.origin - camera_properties.direction * LINEAR_SPEED,
                    ),
                    ..Default::default()
                });
            } else if self.input.key_held(winit::keyboard::KeyCode::KeyA) {
                viewport.set_camera_properties(viewport::PartialViewportCameraProperties {
                    direction: Some(camera_properties.direction.rotate_y(ANGULAR_SPEED)),
                    ..Default::default()
                });
            } else if self.input.key_held(winit::keyboard::KeyCode::KeyD) {
                viewport.set_camera_properties(viewport::PartialViewportCameraProperties {
                    direction: Some(camera_properties.direction.rotate_y(-ANGULAR_SPEED)),
                    ..Default::default()
                });
            }
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
