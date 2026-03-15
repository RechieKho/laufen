use crate::adapter::renderer::*;
use crate::app::arch;

use super::viewport;

#[derive(getset::Getters, getset::MutGetters)]
pub struct Engine {
    #[getset(get = "pub", get_mut = "pub")]
    viewport: viewport::Viewport,
    #[getset(get = "pub", get_mut = "pub")]
    consumer: Option<arch::consumer::Consumer>,
    #[getset(get = "pub", get_mut = "pub")]
    provider: Option<arch::provider::Provider>,

    last_process_timestamp: std::time::Instant,
    frame_per_second: u32,
    current_resolution: (u32, u32),
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
        let consumer = Some(
            self.consumer_builder
                .build(p_parameters.consumer_builder_parameters),
        );
        let window_inner_size = viewport.rendering_server().window().inner_size();
        let current_resolution = (window_inner_size.width, window_inner_size.height);

        Ok(Engine {
            viewport,
            last_process_timestamp: std::time::Instant::now(),
            frame_per_second: 0,
            provider,
            consumer,
            current_resolution,
            abort_flag: Default::default(),
        })
    }
}

impl Engine {
    fn poll_handle_consumer(
        &mut self,
        p_input: &winit_input_helper::WinitInputHelper,
    ) -> anyhow::Result<()> {
        if let Some(consumer) = self.consumer.as_mut() {
            if let Some(notice) = consumer.take_preparation_notice() {
                let builder = viewport::grid_texture_atlas::GridTextureAtlasBuilder {
                    images: &notice.geosphere_texture_images,
                    texture_label: Some("Geosphere texture images"),
                    ..Default::default()
                };
                self.viewport.update_texture_atlas(builder)?;
            }

            consumer.poll_handle_input(p_input, self.current_resolution);
            let camera_spatial = consumer.camera_spatial();
            self.viewport
                .set_camera_properties(viewport::PartialViewportCameraProperties {
                    origin: Some(camera_spatial.origin),
                    direction: Some(camera_spatial.direction),
                    ..Default::default()
                });
        }

        Ok(())
    }

    fn close(&mut self, p_event_loop: &winit::event_loop::ActiveEventLoop) {
        self.abort_flag
            .store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(consumer) = self.consumer.as_ref() {
            consumer.shut_down();
        }
        if let Some(provider) = self.provider.as_ref() {
            provider.shut_down();
        }
        p_event_loop.exit();
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        let quad_instances = if let Some(consumer) = self.consumer.as_mut() {
            consumer.spectate(self.abort_flag.clone(), &self.viewport.camera_properties())
        } else {
            Vec::default()
        };

        let info_text = format!(
            "FPS: {}\nCamera Position: {}",
            self.frame_per_second,
            self.viewport.camera_properties().origin
        );
        let info_text = text::Text::new(info_text.as_str())
            .with_scale(24.0)
            .with_color([1.0, 1.0, 1.0, 1.0]);
        let info_section = text::TextSection::default().add_text(info_text);

        self.viewport.render(viewport::ViewportRenderParameters {
            quad_instances: quad_instances.as_slice(),
            text_sections: &[info_section],
        })
    }

    pub fn process(
        &mut self,
        p_input: &winit_input_helper::WinitInputHelper,
        p_event_loop: &winit::event_loop::ActiveEventLoop,
    ) {
        let current_process_timestamp = std::time::Instant::now();
        let delta = current_process_timestamp.duration_since(self.last_process_timestamp);
        self.last_process_timestamp = current_process_timestamp;
        self.frame_per_second = (1.0 / delta.as_secs_f32()) as _;

        if p_input.close_requested() {
            self.close(p_event_loop);
            return;
        }

        if let Some((width, height)) = p_input.resolution() {
            self.viewport.resize(width, height);
            self.current_resolution = (width, height);
        }

        self.poll_handle_consumer(p_input).unwrap();
    }
}
