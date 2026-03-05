use crate::adapter::renderer;
use crate::adapter::renderer::*;

use super::geosphere;
use super::viewport;

#[derive(getset::Getters, getset::MutGetters)]
pub struct Engine {
    #[getset(get = "pub", get_mut = "pub")]
    viewport: viewport::Viewport,

    #[getset(get = "pub")]
    shared_geosphere: geosphere::SharedGeosphere,

    last_process_timestamp: std::time::Instant,
    frame_per_second: u32,
    spectator: geosphere::spectator::Spectator,
}

#[derive(partially::Partial)]
#[partially(derive(Default))]
pub struct EngineBuilder {
    pub texture_atlas_cell_size: std::num::NonZeroU32,
}

pub struct EngineBuilderParameters<'a> {
    pub event_loop: &'a winit::event_loop::ActiveEventLoop,
}

impl EngineBuilder {
    pub fn try_default() -> anyhow::Result<Self> {
        Ok(Self {
            texture_atlas_cell_size: std::num::NonZeroU32::new(24).unwrap(),
        })
    }

    pub fn build<'a>(self, p_parameters: EngineBuilderParameters<'a>) -> anyhow::Result<Engine> {
        let geosphere = geosphere::GeosphereBuilder::try_with_sample()?.build();
        let geosphere_texture_images = geosphere
            .cube_registry()
            .geosphere_serializable_texture_image()
            .iter()
            .map(|p_image| p_image.clone().to_texture_image().unwrap())
            .collect::<Vec<renderer::texture::TextureImage>>();

        let atlas_builder = viewport::grid_texture_atlas::GridTextureAtlasBuilder {
            cell_size: self.texture_atlas_cell_size,
            images: geosphere_texture_images.as_slice(),
            texture_label: Some("Cube texture atlas"),
        };

        let viewport = viewport::Viewport::new(p_parameters.event_loop, atlas_builder)?;
        let shared_geosphere = geosphere::SharedGeosphere::from(geosphere);

        Ok(Engine {
            viewport,
            shared_geosphere,
            last_process_timestamp: std::time::Instant::now(),
            frame_per_second: 0,
            spectator: geosphere::spectator::Spectator::default(),
        })
    }
}

impl Engine {
    pub fn render(&mut self) -> anyhow::Result<()> {
        let quad_instances = self.spectator.spectate_geosphere(
            self.shared_geosphere.clone(),
            &self.viewport.camera_properties(),
        );

        let frame_per_second_text = format!("FPS: {}", self.frame_per_second);
        let text = text::Text::new(frame_per_second_text.as_str())
            .with_scale(24.0)
            .with_color([1.0, 1.0, 1.0, 1.0]);
        let section = text::TextSection::default().add_text(text);

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
            p_event_loop.exit();
            return;
        }

        if let Some((width, height)) = p_input.resolution() {
            self.viewport.resize(width, height);
        }

        let camera_properties = self.viewport.camera_properties();

        let purge_origin = glam::IVec3::new(
            camera_properties.origin.x as _,
            camera_properties.origin.y as _,
            camera_properties.origin.z as _,
        );
        let purge_distance =
            ((camera_properties.z_far - camera_properties.z_near).abs() * 5.0) as u32;

        self.spectator
            .purge_cache_beyond(purge_origin, purge_distance);
        self.shared_geosphere
            .lock()
            .unwrap()
            .purge_beyond(purge_origin, purge_distance);

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
