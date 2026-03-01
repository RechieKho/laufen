use crate::adapter::renderer::*;

use super::viewport;
use super::viewport::quad;
use super::world;

#[derive(getset::Getters, getset::MutGetters)]
pub struct Engine {
    #[getset(get = "pub", get_mut = "pub")]
    viewport: viewport::Viewport,

    #[getset(get = "pub")]
    cube_registry: world::cube::CubeRegistry,

    #[getset(get = "pub")]
    world: world::World,

    last_process_timestamp: std::time::Instant,
    frame_per_second: u32,
}

#[derive(partially::Partial)]
#[partially(derive(Default))]
pub struct EngineBuilder {
    pub texture_atlas_cell_size: std::num::NonZeroU32,
    pub cube_registry_builder: world::cube::CubeRegistryBuilder,
}

pub struct EngineBuilderParameters<'a> {
    pub event_loop: &'a winit::event_loop::ActiveEventLoop,
}

impl EngineBuilder {
    pub fn try_default() -> anyhow::Result<Self> {
        Ok(Self {
            texture_atlas_cell_size: std::num::NonZeroU32::new(24).unwrap(),
            cube_registry_builder: world::cube::CubeRegistryBuilder::with_builtin()?,
        })
    }

    pub fn build<'a>(self, p_parameters: EngineBuilderParameters<'a>) -> anyhow::Result<Engine> {
        let cube_registry = self.cube_registry_builder.build();

        let atlas_builder = viewport::grid_texture_atlas::GridTextureAtlasBuilder {
            cell_size: self.texture_atlas_cell_size,
            images: cube_registry.world_texture_images().as_slice(),
            texture_label: Some("Cube texture atlas"),
        };

        let viewport = viewport::Viewport::new(p_parameters.event_loop, atlas_builder)?;

        let world = world::World::with_sample_loader_saver();

        Ok(Engine {
            viewport,
            world,
            cube_registry,
            last_process_timestamp: std::time::Instant::now(),
            frame_per_second: 0,
        })
    }
}

impl Engine {
    pub fn render_world<Q: parry3d::query::PointQuery>(
        &mut self,
        p_point_cluster: world::point_cluster::PointCluster,
        p_point_query: &Q,
        p_pose: &parry3d::math::Pose3,
    ) -> anyhow::Result<()> {
        let mut quad_instances = Vec::<quad::QuadInstance>::default();

        for point in p_point_cluster.into_iter() {
            if !p_point_query.contains_point(
                p_pose,
                parry3d::math::Vec3::new(point.x as _, point.y as _, point.z as _),
            ) {
                continue;
            }

            let slot = self.world.slot(point);

            if slot.cube_instance.is_none() {
                continue;
            }

            let cube_instance = slot.cube_instance.unwrap();
            let cube_id = cube_instance.id;
            let cube = self.cube_registry.cubes().get(&cube_id);
            if cube.is_none() {
                continue;
            }
            let cube = cube.unwrap();
            let cube_transformation = glam::Mat4::from(cube_instance.orientation);

            if slot.display_back_quad {
                let back_quad = quad::QuadInstance::new(
                    cube_transformation
                        * quad::QuadTransformationMatrix::from_translation(glam::Vec3::new(
                            0.0 + point.x as f32,
                            0.0 + point.y as f32,
                            0.5 + point.z as f32,
                        ))
                        * quad::QuadRenderPipelineContext::QUAD_BACKWARD_MATRIX,
                    cube.world_texture_atlas_index_back,
                );
                quad_instances.push(back_quad);
            }

            if slot.display_front_quad {
                let front_quad = quad::QuadInstance::new(
                    cube_transformation
                        * quad::QuadTransformationMatrix::from_translation(glam::Vec3::new(
                            0.0 + point.x as f32,
                            0.0 + point.y as f32,
                            -0.5 + point.z as f32,
                        ))
                        * quad::QuadRenderPipelineContext::QUAD_FORWARD_MATRIX,
                    cube.world_texture_atlas_index_front,
                );
                quad_instances.push(front_quad);
            }

            if slot.display_upward_quad {
                let up_quad = quad::QuadInstance::new(
                    cube_transformation
                        * quad::QuadTransformationMatrix::from_translation(glam::Vec3::new(
                            0.0 + point.x as f32,
                            0.5 + point.y as f32,
                            0.0 + point.z as f32,
                        ))
                        * quad::QuadRenderPipelineContext::QUAD_UPWARD_MATRIX,
                    cube.world_texture_atlas_index_top,
                );
                quad_instances.push(up_quad);
            }

            if slot.display_downward_quad {
                let down_quad = quad::QuadInstance::new(
                    cube_transformation
                        * quad::QuadTransformationMatrix::from_translation(glam::Vec3::new(
                            0.0 + point.x as f32,
                            -0.5 + point.y as f32,
                            0.0 + point.z as f32,
                        ))
                        * quad::QuadRenderPipelineContext::QUAD_DOWNWARD_MATRIX,
                    cube.world_texture_atlas_index_bottom,
                );
                quad_instances.push(down_quad);
            }

            if slot.display_left_quad {
                let left_quad = quad::QuadInstance::new(
                    cube_transformation
                        * quad::QuadTransformationMatrix::from_translation(glam::Vec3::new(
                            -0.5 + point.x as f32,
                            0.0 + point.y as f32,
                            0.0 + point.z as f32,
                        ))
                        * quad::QuadRenderPipelineContext::QUAD_LEFT_MATRIX,
                    cube.world_texture_atlas_index_left,
                );
                quad_instances.push(left_quad);
            }

            if slot.display_right_quad {
                let right_quad = quad::QuadInstance::new(
                    cube_transformation
                        * quad::QuadTransformationMatrix::from_translation(glam::Vec3::new(
                            0.5 + point.x as f32,
                            0.0 + point.y as f32,
                            0.0 + point.z as f32,
                        ))
                        * quad::QuadRenderPipelineContext::QUAD_RIGHT_MATRIX,
                    cube.world_texture_atlas_index_right,
                );
                quad_instances.push(right_quad);
            }
        }

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

    pub fn render(&mut self) -> anyhow::Result<()> {
        let camera_properties = self.viewport.camera_properties();
        let cone_height = (camera_properties.z_far - camera_properties.z_near).abs();
        let cone_radius = cone_height * (camera_properties.fov / 3.0).tan();
        let cone = parry3d::shape::Cone::new(cone_height / 2.0, cone_radius);

        let pose = parry3d::math::Pose3::from_mat4(
            glam::Mat4::look_to_rh(
                camera_properties.origin
                    + camera_properties.direction.normalize() * camera_properties.z_near,
                camera_properties.direction,
                glam::Vec3::Y,
            )
            .inverse()
                * glam::Mat4::from_rotation_x(std::f32::consts::PI / 2.0),
        );
        let aabb = cone.aabb(&pose);
        let point_cluster = world::point_cluster::PointCluster::from(aabb);

        self.render_world(point_cluster, &cone, &pose)
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
