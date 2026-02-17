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
        })
    }
}

impl Engine {
    pub fn render_sample_quads(&mut self) -> anyhow::Result<(), wgpu::SurfaceError> {
        // Just render some quads for now.
        self.viewport.render(viewport::ViewportRenderParameters {
            quad_instances: &[
                quad::QuadInstance::new(
                    quad::QuadRenderPipelineContext::QUAD_BACKWARD_MATRIX
                        * quad::QuadTransformationMatrix::from_translation(glam::Vec3::new(
                            -1.0, 0.0, 0.0,
                        )),
                    0,
                ),
                quad::QuadInstance::new(
                    quad::QuadRenderPipelineContext::QUAD_BACKWARD_MATRIX
                        * quad::QuadTransformationMatrix::from_translation(glam::Vec3::new(
                            1.0, 0.0, 0.0,
                        )),
                    1,
                ),
                quad::QuadInstance::new(
                    quad::QuadRenderPipelineContext::QUAD_BACKWARD_MATRIX
                        * quad::QuadTransformationMatrix::from_translation(glam::Vec3::new(
                            2.0, 0.0, 0.0,
                        )),
                    2,
                ),
                quad::QuadInstance::new(
                    quad::QuadRenderPipelineContext::QUAD_BACKWARD_MATRIX
                        * quad::QuadTransformationMatrix::from_translation(glam::Vec3::new(
                            3.0, 0.0, 0.0,
                        )),
                    10,
                ),
            ],
        })
    }

    pub fn render(&mut self) -> anyhow::Result<(), wgpu::SurfaceError> {
        let mut quad_instances = Vec::<quad::QuadInstance>::default();

        self.world.foreach_intersected(
            &world::uaabb::UAabb {
                min: glam::UVec3::new(0, 0, 0),
                max: glam::UVec3::new(20, 20, 20),
            },
            |p_point, p_slot| {
                if p_slot.cube_instance.is_none() {
                    return;
                }

                let cube_instance = p_slot.cube_instance.unwrap();
                let cube_id = cube_instance.id;
                let cube = self.cube_registry.cubes().get(&cube_id);
                if cube.is_none() {
                    return;
                }
                let cube = cube.unwrap();
                let cube_transformation = glam::Mat4::from(cube_instance.orientation);

                let back_quad = quad::QuadInstance::new(
                    cube_transformation
                        * quad::QuadTransformationMatrix::from_translation(glam::Vec3::new(
                            0.0 + p_point.x as f32,
                            0.0 + p_point.y as f32,
                            0.5 + p_point.z as f32,
                        ))
                        * quad::QuadRenderPipelineContext::QUAD_BACKWARD_MATRIX,
                    cube.world_texture_atlas_index_back,
                );
                let front_quad = quad::QuadInstance::new(
                    cube_transformation
                        * quad::QuadTransformationMatrix::from_translation(glam::Vec3::new(
                            0.0 + p_point.x as f32,
                            0.0 + p_point.y as f32,
                            -0.5 + p_point.z as f32,
                        ))
                        * quad::QuadRenderPipelineContext::QUAD_FORWARD_MATRIX,
                    cube.world_texture_atlas_index_front,
                );
                let up_quad = quad::QuadInstance::new(
                    cube_transformation
                        * quad::QuadTransformationMatrix::from_translation(glam::Vec3::new(
                            0.0 + p_point.x as f32,
                            0.5 + p_point.y as f32,
                            0.0 + p_point.z as f32,
                        ))
                        * quad::QuadRenderPipelineContext::QUAD_UPWARD_MATRIX,
                    cube.world_texture_atlas_index_top,
                );
                let down_quad = quad::QuadInstance::new(
                    cube_transformation
                        * quad::QuadTransformationMatrix::from_translation(glam::Vec3::new(
                            0.0 + p_point.x as f32,
                            -0.5 + p_point.y as f32,
                            0.0 + p_point.z as f32,
                        ))
                        * quad::QuadRenderPipelineContext::QUAD_DOWNWARD_MATRIX,
                    cube.world_texture_atlas_index_bottom,
                );
                let left_quad = quad::QuadInstance::new(
                    cube_transformation
                        * quad::QuadTransformationMatrix::from_translation(glam::Vec3::new(
                            -0.5 + p_point.x as f32,
                            0.0 + p_point.y as f32,
                            0.0 + p_point.z as f32,
                        ))
                        * quad::QuadRenderPipelineContext::QUAD_LEFT_MATRIX,
                    cube.world_texture_atlas_index_left,
                );
                let right_quad = quad::QuadInstance::new(
                    cube_transformation
                        * quad::QuadTransformationMatrix::from_translation(glam::Vec3::new(
                            0.5 + p_point.x as f32,
                            0.0 + p_point.y as f32,
                            0.0 + p_point.z as f32,
                        ))
                        * quad::QuadRenderPipelineContext::QUAD_RIGHT_MATRIX,
                    cube.world_texture_atlas_index_right,
                );

                quad_instances.push(back_quad);
                quad_instances.push(front_quad);
                quad_instances.push(left_quad);
                quad_instances.push(right_quad);
                quad_instances.push(up_quad);
                quad_instances.push(down_quad);
            },
        );

        self.viewport.render(viewport::ViewportRenderParameters {
            quad_instances: quad_instances.as_slice(),
        })
    }

    pub fn process(
        &mut self,
        p_input: &winit_input_helper::WinitInputHelper,
        p_event_loop: &winit::event_loop::ActiveEventLoop,
    ) {
        if p_input.close_requested() {
            p_event_loop.exit();
            return;
        }

        if let Some((width, height)) = p_input.resolution() {
            self.viewport.resize(width, height);
        }

        let camera_properties = self.viewport.camera_properties();
        const LINEAR_SPEED: f32 = 0.05;
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
        }
    }
}
