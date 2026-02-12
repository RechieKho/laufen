use super::buffer::ToBuffer;

use super::buffer;
use super::index_buffer;
use super::rendering_server;
use super::vertex_buffer;

#[derive(Default)]
pub struct RenderData {
    pub vertex_buffers: Vec<vertex_buffer::VertexBuffer>,
    pub index_buffer: Option<index_buffer::IndexBuffer>,
    pub vertex_buffer_slot_offset: u32,
}

impl rendering_server::SubmitToRenderPass for RenderData {
    fn submit<'a>(&self, p_render_pass: &mut wgpu::RenderPass<'a>) {
        for (i, buffer) in self.vertex_buffers.iter().enumerate() {
            p_render_pass.set_vertex_buffer(
                i as u32 + self.vertex_buffer_slot_offset,
                (*buffer).slice(..),
            );
        }
        if let Some(buffer) = &self.index_buffer {
            buffer.submit(p_render_pass);
        }
    }
}

impl RenderData {
    pub fn add_vertex_collections<V>(
        &mut self,
        p_server: &rendering_server::RenderingServer,
        p_vertex_collections: &[&[V]],
    ) where
        [V]: buffer::ToBuffer<Output = vertex_buffer::VertexBuffer>,
    {
        self.vertex_buffers.extend(
            p_vertex_collections
                .iter()
                .map(|vertex_group| vertex_group.to_buffer(p_server, None)),
        );
    }

    pub fn set_indices<I>(
        &mut self,
        p_server: &rendering_server::RenderingServer,
        p_indices: Option<&[I]>,
    ) where
        [I]: buffer::ToBuffer<Output = index_buffer::IndexBuffer>,
    {
        self.index_buffer = p_indices.map(|indices| indices.to_buffer(p_server, None));
    }

    pub fn clear(&mut self) {
        self.vertex_buffers.clear();
        self.index_buffer = None;
    }
}
