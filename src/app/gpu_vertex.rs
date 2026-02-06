
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GPUVertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

pub const TRIANGLE_VERTICES: &[GPUVertex] = &[
    GPUVertex { position: [0.0, 0.5, 0.0], color: [1.0, 0.0, 0.0] },
    GPUVertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] },
    GPUVertex { position: [0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0] },
];

impl GPUVertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

    pub fn describe() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

