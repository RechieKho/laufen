use super::rendering_server;

/// Convert to Buffer.
pub trait ToBuffer {
    type Output;
    fn to_buffer(
        &self,
        p_server: &rendering_server::RenderingServer,
        p_label: Option<&str>,
    ) -> Self::Output;
}

pub trait BufferElement: repr_trait::C + bytemuck::Pod + bytemuck::Zeroable {}
