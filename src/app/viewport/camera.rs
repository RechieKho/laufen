use crate::adapter::renderer::bind_group::ToBindGroupContext;
use crate::adapter::renderer::buffer::ToBuffer;
use crate::adapter::renderer::*;
use repr_trait::C;

#[derive(getset::Getters, getset::MutGetters)]
pub struct CameraContext {
    pub properties: CameraProperties,

    #[getset(get = "pub", get_mut = "pub")]
    uniform_buffer: uniform_buffer::UniformBuffer,
    #[getset(get = "pub", get_mut = "pub")]
    bind_group_context: bind_group::BindGroupContext,
}

#[derive(partially::Partial)]
#[partially(derive(Default))]
pub struct CameraProperties {
    pub origin: glam::Vec3,
    pub direction: glam::Vec3,
    pub up: glam::Vec3,
    pub aspect_ratio: f32,
    pub fov_y: f32,
    pub z_near: f32,
    pub z_far: f32,
}

pub type TransformationMatrix = glam::Mat4;
pub type RawTransformationMatrix = [[f32; 4]; 4];

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, repr_trait::C)]
pub struct CameraUniform {
    pub transformation_matrix: RawTransformationMatrix,
}

impl buffer::BufferElement for CameraUniform {}
impl uniform_buffer::UniformBufferElement for CameraUniform {}

impl Default for CameraProperties {
    fn default() -> Self {
        Self {
            origin: glam::Vec3::new(0.0, 1.0, 5.0),
            direction: glam::Vec3::NEG_Z,
            up: glam::Vec3::Y,
            aspect_ratio: 16.0 / 9.0,
            fov_y: std::f32::consts::PI / 4.0,
            z_near: 0.1,
            z_far: 200.0,
        }
    }
}

impl CameraProperties {
    pub fn create_transform(&self) -> CameraUniform {
        let projection =
            glam::Mat4::perspective_rh(self.fov_y, self.aspect_ratio, self.z_near, self.z_far);
        let view = glam::Mat4::look_to_rh(self.origin, self.direction, self.up);
        CameraUniform {
            transformation_matrix: bytemuck::cast(projection * view),
        }
    }
}

impl CameraContext {
    pub fn new(p_server: &rendering_server::RenderingServer) -> Self {
        let properties = CameraProperties::default();
        let transform = properties.create_transform();
        let uniform_buffer = uniform_buffer::ToUniformBuffer(&[transform])
            .to_buffer(p_server, Some("Camera uniform buffer"));
        let bind_group_context =
            uniform_buffer.to_bind_group_context(p_server, Some("Camera bind group"));
        Self {
            properties,
            uniform_buffer,
            bind_group_context,
        }
    }
}

impl rendering_server::SubmitToQueue for CameraContext {
    fn submit(&self, p_queue: &rendering_server::Queue) {
        let new_transform = self.properties.create_transform();
        p_queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[new_transform]),
        );
    }
}
