use crate::adapter::renderer::*;
use crate::adapter::shared;
use crate::app::arch;

use super::viewport;

pub type ConsumerHandle = arch::pollable::PollHandle<arch::consumer::Consumer>;

#[derive(getset::Getters, getset::MutGetters)]
pub struct Engine {
    #[getset(get = "pub", get_mut = "pub")]
    viewport: viewport::Viewport,
    #[getset(get = "pub", get_mut = "pub")]
    consumer_handle: Option<ConsumerHandle>,
    #[getset(get = "pub", get_mut = "pub")]
    provider: Option<arch::provider::Provider>,

    last_process_timestamp: std::time::Instant,
    frame_per_second: u32,
    abort_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

#[derive(partially::Partial)]
#[partially(derive(Default))]
pub struct UnsecureEngineBuilder {
    pub provider_builder: arch::provider::ProviderBuilder,
    pub consumer_builder: arch::consumer::UnsecureConsumerBuilder,
    pub poll_interval_duration: std::time::Duration,
}

pub struct UnsecureEngineBuilderParameters<'a> {
    pub event_loop: &'a winit::event_loop::ActiveEventLoop,
    pub provider_builder_parameters: arch::provider::ProviderBuilderParameters,
    pub consumer_builder_parameters: arch::consumer::UnsecureConsumerBuilderParameters,
}

impl UnsecureEngineBuilder {
    pub fn try_with_sample() -> anyhow::Result<Self> {
        Ok(Self {
            provider_builder: arch::provider::ProviderBuilder::try_with_sample()?,
            consumer_builder: arch::consumer::UnsecureConsumerBuilder::default(),
            poll_interval_duration: std::time::Duration::from_millis(16),
        })
    }

    pub fn build<'a>(
        self,
        p_parameters: UnsecureEngineBuilderParameters<'a>,
    ) -> anyhow::Result<Engine> {
        let viewport = viewport::Viewport::new(p_parameters.event_loop)?;
        let provider = Some(
            self.provider_builder
                .build(p_parameters.provider_builder_parameters),
        );
        let consumer = shared::share(
            self.consumer_builder
                .build(p_parameters.consumer_builder_parameters),
        );
        let consumer_handle = Some(ConsumerHandle::new(consumer, self.poll_interval_duration));

        Ok(Engine {
            viewport,
            last_process_timestamp: std::time::Instant::now(),
            frame_per_second: 0,
            provider,
            consumer_handle,
            abort_flag: Default::default(),
        })
    }
}

impl Engine {
    fn handle_consumer_preparation_notice(&mut self) -> anyhow::Result<()> {
        if let Some(consumer_handle) = self.consumer_handle.as_ref() {
            let mut consumer = consumer_handle.shared_pollable().lock().unwrap();
            if let Some(notice) = consumer.take_preparation_notice() {
                if !notice.geosphere_texture_images.is_empty() {
                    let builder = viewport::grid_texture_atlas::GridTextureAtlasBuilder {
                        images: &notice.geosphere_texture_images,
                        texture_label: Some("Geosphere texture images"),
                        ..Default::default()
                    };
                    self.viewport.update_texture_atlas(builder)?;
                }
            }
        }

        Ok(())
    }

    fn close(&mut self, p_event_loop: &winit::event_loop::ActiveEventLoop) {
        self.abort_flag
            .store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(handle) = self.consumer_handle.as_ref() {
            handle.shut_down();
            handle.shared_pollable().lock().unwrap().disconnect();
        }
        if let Some(provider) = self.provider.as_ref() {
            provider.shut_down();
        }
        p_event_loop.exit();
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        let quad_instances = if let Some(handle) = self.consumer_handle.as_ref() {
            handle
                .shared_pollable()
                .lock()
                .unwrap()
                .spectate(self.abort_flag.clone(), &self.viewport.camera_properties())
        } else {
            Vec::default()
        };

        let frame_per_second_text = format!("FPS: {}", self.frame_per_second);
        let frame_per_second_text = text::Text::new(frame_per_second_text.as_str())
            .with_scale(24.0)
            .with_color([1.0, 1.0, 1.0, 1.0]);
        let section = text::TextSection::default().add_text(frame_per_second_text);

        self.viewport.render(viewport::ViewportRenderParameters {
            quad_instances: quad_instances.as_slice(),
            text_sections: &[section],
        })
    }

    pub fn process(
        &mut self,
        p_input: &winit_input_helper::WinitInputHelper,
        p_event_loop: &winit::event_loop::ActiveEventLoop,
    ) {
        let current_process_timestamp = std::time::Instant::now();
        let delta = current_process_timestamp.duration_since(self.last_process_timestamp);
        self.frame_per_second = (1.0 / delta.as_secs_f32()) as _;

        if p_input.close_requested() {
            self.close(p_event_loop);
            return;
        }

        if let Some((width, height)) = p_input.resolution() {
            self.viewport.resize(width, height);
        }

        self.handle_consumer_preparation_notice().unwrap();

        let camera_properties = self.viewport.camera_properties();

        const LINEAR_SPEED: f32 = 0.2;
        const ANGULAR_SPEED: f32 = std::f32::consts::PI / 100.0;

        if p_input.key_held(winit::keyboard::KeyCode::KeyW) {
            self.viewport
                .set_camera_properties(viewport::PartialViewportCameraProperties {
                    origin: Some(
                        camera_properties.origin + camera_properties.direction * LINEAR_SPEED,
                    ),
                    ..Default::default()
                });
        } else if p_input.key_held(winit::keyboard::KeyCode::KeyS) {
            self.viewport
                .set_camera_properties(viewport::PartialViewportCameraProperties {
                    origin: Some(
                        camera_properties.origin - camera_properties.direction * LINEAR_SPEED,
                    ),
                    ..Default::default()
                });
        } else if p_input.key_held(winit::keyboard::KeyCode::KeyA) {
            self.viewport
                .set_camera_properties(viewport::PartialViewportCameraProperties {
                    direction: Some(camera_properties.direction.rotate_y(ANGULAR_SPEED)),
                    ..Default::default()
                });
        } else if p_input.key_held(winit::keyboard::KeyCode::KeyD) {
            self.viewport
                .set_camera_properties(viewport::PartialViewportCameraProperties {
                    direction: Some(camera_properties.direction.rotate_y(-ANGULAR_SPEED)),
                    ..Default::default()
                });
        } else if p_input.key_held(winit::keyboard::KeyCode::KeyQ) {
            self.viewport
                .set_camera_properties(viewport::PartialViewportCameraProperties {
                    origin: Some(camera_properties.origin - glam::Vec3::Y * LINEAR_SPEED),
                    ..Default::default()
                });
        } else if p_input.key_held(winit::keyboard::KeyCode::KeyE) {
            self.viewport
                .set_camera_properties(viewport::PartialViewportCameraProperties {
                    origin: Some(camera_properties.origin - glam::Vec3::NEG_Y * LINEAR_SPEED),
                    ..Default::default()
                });
        }

        self.last_process_timestamp = current_process_timestamp;
    }
}
