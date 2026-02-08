use super::buffer::ToBuffer;

use super::buffer;
use super::index_buffer;
use super::rendering_server;
use super::vertex_buffer;

pub struct RenderData {
    pub vertex_buffers: Vec<vertex_buffer::VertexBuffer>,
    pub index_buffer: Option<index_buffer::IndexBuffer>,
}

impl rendering_server::SubmitToRenderPass for RenderData {
    fn submit<'a>(&self, p_render_pass: &mut wgpu::RenderPass<'a>) {
        self.vertex_buffers.as_slice().submit(p_render_pass);
        if let Some(buffer) = &self.index_buffer {
            buffer.submit(p_render_pass);
        }
    }
}

impl RenderData {
    pub fn compile<T, I>(
        p_server: &rendering_server::RenderingServer,
        p_vertices: &[&[T]],
        p_indices: Option<&[I]>,
    ) -> RenderData
    where
        [T]: buffer::ToBuffer<Output = vertex_buffer::VertexBuffer>,
        [I]: buffer::ToBuffer<Output = index_buffer::IndexBuffer>,
    {
        let vertex_buffers = p_vertices
            .iter()
            .map(|vertex_group| vertex_group.to_buffer(p_server, None))
            .collect::<Vec<vertex_buffer::VertexBuffer>>();

        RenderData {
            vertex_buffers,
            index_buffer: p_indices.map(|indices| indices.to_buffer(p_server, None)),
        }
    }
}
