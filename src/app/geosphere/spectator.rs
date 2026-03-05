use crate::app::viewport;
use crate::app::viewport::quad;
use parry3d::query::PointQuery;

use super::cube;
use super::point_cluster;
use super::slot;

type Lump = rapidhash::RapidHashMap<glam::IVec3, Vec<quad::QuadInstance>>;
type SharedLump = std::sync::Arc<std::sync::Mutex<Lump>>;

type LoadingLump = rapidhash::RapidHashSet<glam::IVec3>;
type SharedLoadingLump = std::sync::Arc<std::sync::Mutex<LoadingLump>>;

pub type SlotCubeProxy = std::sync::Arc<
    std::sync::Mutex<
        dyn Fn(glam::IVec3) -> (slot::Slot, Option<cube::Cube>) + Send + Sync + 'static,
    >,
>;

#[derive(Default)]
pub struct Spectator {
    lump: SharedLump,
    loading_lump: SharedLoadingLump,
}

impl Spectator {
    pub const LUMP_SIZE: u32 = 8;
    pub const MAX_LOAD_PER_SPECTATE: u32 = 128;

    #[inline]
    fn convert_slot_position_to_lump_position(p_slot_position: glam::IVec3) -> glam::IVec3 {
        p_slot_position.div_euclid(glam::IVec3::splat(Self::LUMP_SIZE as _))
    }

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

    fn create_quad_instances_from_slot(
        p_slot_cube_proxy: SlotCubeProxy,
        p_slot_position: glam::IVec3,
    ) -> Vec<quad::QuadInstance> {
        let (slot, cube) = p_slot_cube_proxy.lock().unwrap()(p_slot_position);

        if cube.is_none() {
            return Default::default();
        }

        if slot.cube_instance.is_none() {
            return Default::default();
        }

        let cube_instance = slot.cube_instance.unwrap();
        let cube = cube.unwrap();
        let cube_transformation = glam::Mat4::from(cube_instance.orientation);
        let mut instances = Vec::<quad::QuadInstance>::default();

        if slot.display_back_quad {
            let back_quad = quad::QuadInstance::new(
                cube_transformation
                    * quad::QuadTransformationMatrix::from_translation(glam::Vec3::new(
                        0.0 + p_slot_position.x as f32,
                        0.0 + p_slot_position.y as f32,
                        0.5 + p_slot_position.z as f32,
                    ))
                    * quad::QuadRenderPipelineContext::QUAD_BACKWARD_MATRIX,
                cube.geosphere_texture_atlas_index_back,
            );
            instances.push(back_quad);
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
                cube.geosphere_texture_atlas_index_front,
            );
            instances.push(front_quad);
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
                cube.geosphere_texture_atlas_index_top,
            );
            instances.push(up_quad);
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
                cube.geosphere_texture_atlas_index_bottom,
            );
            instances.push(down_quad);
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
                cube.geosphere_texture_atlas_index_left,
            );
            instances.push(left_quad);
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
                cube.geosphere_texture_atlas_index_right,
            );
            instances.push(right_quad);
        }

        instances
    }

    fn load_to_lump(
        p_slot_cube_proxy: SlotCubeProxy,
        p_lump_position: glam::IVec3,
        r_shared_lump: SharedLump,
    ) {
        let mut instances = Vec::<quad::QuadInstance>::default();
        for slot_point in Self::compute_slot_point_cluster_from_lump_position(p_lump_position) {
            instances.append(&mut Self::create_quad_instances_from_slot(
                p_slot_cube_proxy.clone(),
                slot_point,
            ));
        }
        r_shared_lump
            .lock()
            .unwrap()
            .insert(p_lump_position, instances);
    }

    fn load_to_lump_threaded(
        p_slot_cube_proxy: SlotCubeProxy,
        p_lump_position: glam::IVec3,
        m_loading_lump: SharedLoadingLump,
        r_shared_lump: SharedLump,
    ) -> Option<tokio::task::JoinHandle<()>> {
        if m_loading_lump.lock().unwrap().contains(&p_lump_position) {
            return None;
        }

        m_loading_lump.lock().unwrap().insert(p_lump_position);
        Some(tokio::task::spawn_blocking(move || {
            p_slot_cube_proxy.lock().unwrap()(glam::IVec3::ZERO);
            Self::load_to_lump(p_slot_cube_proxy, p_lump_position, r_shared_lump);
            m_loading_lump.lock().unwrap().remove(&p_lump_position);
        }))
    }

    pub fn spectate(
        &mut self,
        p_slot_cube_proxy: SlotCubeProxy,
        p_camera_properties: &viewport::ViewportCameraProperties,
    ) -> Vec<quad::QuadInstance> {
        let lump_cone_height =
            (p_camera_properties.z_far - p_camera_properties.z_near).abs() / Self::LUMP_SIZE as f32;
        let lump_cone_radius = (lump_cone_height * p_camera_properties.fov.tan()
            / Self::LUMP_SIZE as f32)
            + Self::LUMP_SIZE as f32;
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
        let mut load_count = 0u32;

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

            if let Some(instances) = self.lump.lock().unwrap().get(&lump_position) {
                quad_instances.extend_from_slice(instances.as_slice());
            } else {
                if load_count > Self::MAX_LOAD_PER_SPECTATE {
                    continue;
                }
                std::mem::drop(Self::load_to_lump_threaded(
                    p_slot_cube_proxy.clone(),
                    lump_position,
                    self.loading_lump.clone(),
                    self.lump.clone(),
                ));
                load_count += 1;
            }
        }

        quad_instances
    }

    fn create_proxy_from_shared_geosphere(
        p_shared_geosphere: super::SharedGeosphere,
    ) -> SlotCubeProxy {
        std::sync::Arc::new(std::sync::Mutex::new(move |p_position: glam::IVec3| {
            let mut geosphere = p_shared_geosphere.lock().unwrap();
            let (slot, cube) = geosphere.slot_cube(p_position);
            (*slot, cube)
        }))
    }

    pub fn spectate_geosphere(
        &mut self,
        p_shared_geosphere: super::SharedGeosphere,
        p_camera_properties: &viewport::ViewportCameraProperties,
    ) -> Vec<quad::QuadInstance> {
        let proxy = Self::create_proxy_from_shared_geosphere(p_shared_geosphere);
        self.spectate(proxy, p_camera_properties)
    }

    pub fn purge_cache_beyond(
        &mut self,
        p_slot_position: glam::IVec3,
        p_max_chebyshev_slot_distance: u32,
    ) {
        let lump_position = Self::convert_slot_position_to_lump_position(p_slot_position);
        let max_chebyshev_lump_distance =
            p_max_chebyshev_slot_distance.div_euclid(Self::LUMP_SIZE as _);

        self.lump.lock().unwrap().retain(|p_key, _| {
            let distance = lump_position.chebyshev_distance(*p_key);
            distance < max_chebyshev_lump_distance
        })
    }
}
