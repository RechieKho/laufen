use repr_trait::C;
use std::mem;

use super::vertex_buffer;

pub type TransformationMatrix = glam::Mat4;
pub type RawTransformationMatrix = [[f32; 4]; 4];

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, repr_trait::C)]
pub struct Instance {
    pub transformation_matrix: RawTransformationMatrix,
}

const INSTANCE_BUFFER_ATTRIBUTES: [vertex_buffer::VertexAttribute; 4] = vertex_buffer::vertex_attr_array![5 => Float32x4, 6 => Float32x4, 7 => Float32x4, 8 => Float32x4];

impl vertex_buffer::VertexBufferElement for Instance {
    fn get_vertex_buffer_layout() -> vertex_buffer::VertexBufferLayout<'static> {
        vertex_buffer::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: vertex_buffer::VertexStepMode::Instance,
            attributes: &INSTANCE_BUFFER_ATTRIBUTES,
        }
    }
}

impl From<glam::Mat4> for Instance {
    fn from(p_value: glam::Mat4) -> Self {
        Self {
            transformation_matrix: bytemuck::cast(p_value),
        }
    }
}
