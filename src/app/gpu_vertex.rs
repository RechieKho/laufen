
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GPUVertex {
    pub position: [f32; 3],
    pub texture_coordinate: [f32; 2], // NEW!
}

pub const TRIANGLE_VERTICES: &[GPUVertex] = &[
    GPUVertex { position: [0.0, 0.5, 0.0], texture_coordinate: [0.0, 0.0] },
    GPUVertex { position: [-0.5, -0.5, 0.0], texture_coordinate: [0.0, 1.0] },
    GPUVertex { position: [0.5, -0.5, 0.0], texture_coordinate: [1.0, 1.0] },
];

pub const TRIANGLE_INDICES: &[u16] = &[
    1, 2, 3
];

impl GPUVertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    pub fn describe() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

