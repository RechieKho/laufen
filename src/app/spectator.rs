use parry3d::query::PointQuery;

use super::viewport;
use super::viewport::quad;
use super::world;
use super::world::cube;
use super::world::point_cluster;

#[derive(Default)]
pub struct Spectator<const N: u32> {
    lump: rapidhash::RapidHashMap<glam::IVec3, Vec<quad::QuadInstance>>,
}

impl<const N: u32> Spectator<N> {
    pub const LUMP_SIZE: u32 = N;

    #[inline]
    fn convert_lump_position_to_slot_position(p_lump_position: glam::IVec3) -> glam::IVec3 {
        p_lump_position * Self::LUMP_SIZE as i32
    }

    #[inline]
    fn compute_slot_point_cluster_from_lump_position(
        p_lump_position: glam::IVec3,
    ) -> point_cluster::PointCluster {
        point_cluster::PointCluster {
            min: Self::convert_lump_position_to_slot_position(p_lump_position),
            max: Self::convert_lump_position_to_slot_position(p_lump_position + 1),
        }
    }

    fn append_quad_instances_from_slot(
        p_world: &mut world::World,
        p_cube_registry: &cube::CubeRegistry,
        p_slot_position: glam::IVec3,
        r_instances: &mut Vec<quad::QuadInstance>,
    ) {
        let slot = p_world.slot(p_slot_position);

        if slot.cube_instance.is_none() {
            return;
        }

        let cube_instance = slot.cube_instance.unwrap();
        let cube_id = cube_instance.id;
        let cube = p_cube_registry.cubes().get(&cube_id);
        if cube.is_none() {
            return;
        }
        let cube = cube.unwrap();
        let cube_transformation = glam::Mat4::from(cube_instance.orientation);

        if slot.display_back_quad {
            let back_quad = quad::QuadInstance::new(
                cube_transformation
                    * quad::QuadTransformationMatrix::from_translation(glam::Vec3::new(
                        0.0 + p_slot_position.x as f32,
                        0.0 + p_slot_position.y as f32,
                        0.5 + p_slot_position.z as f32,
                    ))
                    * quad::QuadRenderPipelineContext::QUAD_BACKWARD_MATRIX,
                cube.world_texture_atlas_index_back,
            );
            r_instances.push(back_quad);
        }

        if slot.display_front_quad {
            let front_quad = quad::QuadInstance::new(
                cube_transformation
                    * quad::QuadTransformationMatrix::from_translation(glam::Vec3::new(
                        0.0 + p_slot_position.x as f32,
                        0.0 + p_slot_position.y as f32,
                        -0.5 + p_slot_position.z as f32,
                    ))
                    * quad::QuadRenderPipelineContext::QUAD_FORWARD_MATRIX,
                cube.world_texture_atlas_index_front,
            );
            r_instances.push(front_quad);
        }

        if slot.display_upward_quad {
            let up_quad = quad::QuadInstance::new(
                cube_transformation
                    * quad::QuadTransformationMatrix::from_translation(glam::Vec3::new(
                        0.0 + p_slot_position.x as f32,
                        0.5 + p_slot_position.y as f32,
                        0.0 + p_slot_position.z as f32,
                    ))
                    * quad::QuadRenderPipelineContext::QUAD_UPWARD_MATRIX,
                cube.world_texture_atlas_index_top,
            );
            r_instances.push(up_quad);
        }

        if slot.display_downward_quad {
            let down_quad = quad::QuadInstance::new(
                cube_transformation
                    * quad::QuadTransformationMatrix::from_translation(glam::Vec3::new(
                        0.0 + p_slot_position.x as f32,
                        -0.5 + p_slot_position.y as f32,
                        0.0 + p_slot_position.z as f32,
                    ))
                    * quad::QuadRenderPipelineContext::QUAD_DOWNWARD_MATRIX,
                cube.world_texture_atlas_index_bottom,
            );
            r_instances.push(down_quad);
        }

        if slot.display_left_quad {
            let left_quad = quad::QuadInstance::new(
                cube_transformation
                    * quad::QuadTransformationMatrix::from_translation(glam::Vec3::new(
                        -0.5 + p_slot_position.x as f32,
                        0.0 + p_slot_position.y as f32,
                        0.0 + p_slot_position.z as f32,
                    ))
                    * quad::QuadRenderPipelineContext::QUAD_LEFT_MATRIX,
                cube.world_texture_atlas_index_left,
            );
            r_instances.push(left_quad);
        }

        if slot.display_right_quad {
            let right_quad = quad::QuadInstance::new(
                cube_transformation
                    * quad::QuadTransformationMatrix::from_translation(glam::Vec3::new(
                        0.5 + p_slot_position.x as f32,
                        0.0 + p_slot_position.y as f32,
                        0.0 + p_slot_position.z as f32,
                    ))
                    * quad::QuadRenderPipelineContext::QUAD_RIGHT_MATRIX,
                cube.world_texture_atlas_index_right,
            );
            r_instances.push(right_quad);
        }
    }

    pub fn spectate(
        &mut self,
        p_world: &mut world::World,
        p_cube_registry: &cube::CubeRegistry,
        p_camera_properties: &viewport::ViewportCameraProperties,
    ) -> Vec<quad::QuadInstance> {
        let lump_cone_height =
            (p_camera_properties.z_far - p_camera_properties.z_near).abs() / Self::LUMP_SIZE as f32;
        let lump_cone_radius = lump_cone_height
            * (p_camera_properties.fov * Self::LUMP_SIZE as f32 / 4.0).tan()
            / Self::LUMP_SIZE as f32;
        let lump_cone = parry3d::shape::Cone::new(lump_cone_height / 2.0, lump_cone_radius);

        let pose = parry3d::math::Pose3::from_mat4(
            glam::Mat4::look_to_rh(
                (p_camera_properties.origin
                    + p_camera_properties.direction.normalize() * p_camera_properties.z_near)
                    / Self::LUMP_SIZE as f32,
                p_camera_properties.direction,
                glam::Vec3::Y,
            )
            .inverse()
                * glam::Mat4::from_rotation_x(std::f32::consts::PI / 2.0),
        );

        let mut quad_instances = Vec::<quad::QuadInstance>::default();

        for lump_position in point_cluster::PointCluster::from(lump_cone.aabb(&pose)).into_iter() {
            if !lump_cone.contains_point(
                &pose,
                parry3d::math::Vec3::new(
                    lump_position.x as _,
                    lump_position.y as _,
                    lump_position.z as _,
                ),
            ) {
                continue;
            }

            if let Some(instances) = self.lump.get(&lump_position) {
                quad_instances.extend_from_slice(instances.as_slice());
            } else {
                let mut instances = Vec::<quad::QuadInstance>::default();
                for slot_point in Self::compute_slot_point_cluster_from_lump_position(lump_position)
                {
                    Self::append_quad_instances_from_slot(
                        p_world,
                        p_cube_registry,
                        slot_point,
                        &mut instances,
                    );
                }
                quad_instances.extend_from_slice(instances.as_slice());
                self.lump.insert(lump_position, instances);
            }
        }

        quad_instances
    }
}
