use super::buffer;
use super::rendering_server;

pub struct IndexBuffer {
    buffer: rendering_server::Buffer,
    index_format: wgpu::IndexFormat,
}

impl buffer::ToBuffer for [u16] {
    type Output = IndexBuffer;

    fn to_buffer(
        &self,
        p_server: &rendering_server::RenderingServer,
        p_label: Option<&str>,
    ) -> Self::Output {
        IndexBuffer {
            buffer: p_server.create_buffer(&rendering_server::util::BufferInitDescriptor {
                label: p_label,
                contents: bytemuck::cast_slice(self),
                usage: rendering_server::BufferUsages::INDEX,
            }),
            index_format: wgpu::IndexFormat::Uint16,
        }
    }
}

impl buffer::ToBuffer for [u32] {
    type Output = IndexBuffer;

    fn to_buffer(
        &self,
        p_server: &rendering_server::RenderingServer,
        p_label: Option<&str>,
    ) -> Self::Output {
        IndexBuffer {
            buffer: p_server.create_buffer(&rendering_server::util::BufferInitDescriptor {
                label: p_label,
                contents: bytemuck::cast_slice(self),
                usage: rendering_server::BufferUsages::INDEX,
            }),
            index_format: wgpu::IndexFormat::Uint32,
        }
    }
}

impl rendering_server::SubmitToRenderPass for IndexBuffer {
    fn submit<'a>(&self, p_render_pass: &mut rendering_server::RenderPass<'a>) {
        p_render_pass.set_index_buffer(self.buffer.slice(..), self.index_format);
    }
}
