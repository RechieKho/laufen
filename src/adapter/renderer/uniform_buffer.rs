use super::bind_group;
use super::buffer;
use super::rendering_server;

pub trait UniformBufferElement: buffer::BufferElement {}

pub struct UniformBuffer(rendering_server::Buffer);

impl std::ops::Deref for UniformBuffer {
    type Target = rendering_server::Buffer;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for UniformBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct ToUniformBuffer<'a, T>(pub &'a [T])
where
    T: UniformBufferElement;

impl<'a, T> buffer::ToBuffer for ToUniformBuffer<'a, T>
where
    T: UniformBufferElement,
{
    type Output = UniformBuffer;

    fn to_buffer(
        &self,
        p_server: &rendering_server::RenderingServer,
        p_label: Option<&str>,
    ) -> Self::Output {
        UniformBuffer(
            p_server.create_buffer(&rendering_server::util::BufferInitDescriptor {
                label: p_label,
                contents: bytemuck::cast_slice(self.0),
                usage: rendering_server::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
        )
    }
}

impl bind_group::ToBindGroupContext for UniformBuffer {
    const BIND_GROUP_LAYOUT_DESCRIPTOR: rendering_server::BindGroupLayoutDescriptor<'static> =
        rendering_server::BindGroupLayoutDescriptor {
            entries: &[rendering_server::BindGroupLayoutEntry {
                binding: 0,
                visibility: rendering_server::ShaderStages::VERTEX,
                ty: rendering_server::BindingType::Buffer {
                    ty: rendering_server::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("Uniform buffer bind group layout"),
        };

    fn to_bind_group_context(
        &self,
        p_server: &rendering_server::RenderingServer,
        p_label: Option<&str>,
    ) -> bind_group::BindGroupContext {
        let layout = self.create_bind_group_layout(p_server);
        let bind_group = p_server.create_bind_group(&rendering_server::BindGroupDescriptor {
            layout: &layout,
            entries: &[rendering_server::BindGroupEntry {
                binding: 0,
                resource: (*self).as_entire_binding(),
            }],
            label: p_label,
        });

        bind_group::BindGroupContext {
            bind_group,
            bind_group_layout: layout,
            bind_group_offset: 0,
        }
    }
}
