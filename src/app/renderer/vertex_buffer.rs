use repr_trait::C;
use std::mem;

use super::buffer;
use super::rendering_server;

pub trait VertexBufferElement: repr_trait::C + bytemuck::Pod + bytemuck::Zeroable {
    fn get_vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static>;
}

type Position = [f32; 3];
type TextureCoordinate = [f32; 2];

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, repr_trait::C)]
pub struct SimpleVertex {
    pub position: Position,
    pub texture_coordiation: TextureCoordinate,
}
const SIMPLE_VERTEX_BUFFER_ATTRIBUTES: [wgpu::VertexAttribute; 2] =
    wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

impl VertexBufferElement for SimpleVertex {
    fn get_vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &SIMPLE_VERTEX_BUFFER_ATTRIBUTES,
        }
    }
}

pub struct VertexBuffer(rendering_server::Buffer);

impl<T> buffer::ToBuffer for [T]
where
    T: VertexBufferElement,
{
    type Output = VertexBuffer;

    fn to_buffer(
        &self,
        p_server: &super::rendering_server::RenderingServer,
        p_label: Option<&str>,
    ) -> Self::Output {
        VertexBuffer(
            p_server.create_buffer(&rendering_server::BufferInitDescriptor {
                label: p_label,
                contents: bytemuck::cast_slice(self),
                usage: rendering_server::BufferUsages::VERTEX,
            }),
        )
    }
}

impl rendering_server::SubmitToRenderPass for [VertexBuffer] {
    fn submit<'a>(&self, p_render_pass: &mut rendering_server::RenderPass<'a>) {
        for (i, buffer) in self.iter().enumerate() {
            p_render_pass.set_vertex_buffer(i as u32, buffer.0.slice(..));
        }
    }
}
