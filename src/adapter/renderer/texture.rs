use super::bind_group;
use super::rendering_server;

pub struct TextureContext {
    pub texture: rendering_server::Texture,
    pub view: rendering_server::TextureView,
    pub sampler: rendering_server::Sampler,
}

impl bind_group::ToBindGroupContext for TextureContext {
    const BIND_GROUP_LAYOUT_DESCRIPTOR: rendering_server::BindGroupLayoutDescriptor<'static> =
        rendering_server::BindGroupLayoutDescriptor {
            entries: &[
                rendering_server::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: rendering_server::ShaderStages::FRAGMENT,
                    ty: rendering_server::BindingType::Texture {
                        multisampled: false,
                        view_dimension: rendering_server::TextureViewDimension::D2,
                        sample_type: rendering_server::TextureSampleType::Float {
                            filterable: true,
                        },
                    },
                    count: None,
                },
                rendering_server::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: rendering_server::ShaderStages::FRAGMENT,
                    // This should match the filterable field of the
                    // corresponding Texture entry above.
                    ty: rendering_server::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("Texture bind group layout"),
        };

    fn to_bind_group_context(
        &self,
        p_server: &rendering_server::RenderingServer,
        p_label: Option<&str>,
    ) -> bind_group::BindGroupContext {
        let layout = self.create_bind_group_layout(p_server);
        let bind_group = p_server.create_bind_group(&rendering_server::BindGroupDescriptor {
            layout: &layout,
            entries: &[
                rendering_server::BindGroupEntry {
                    binding: 0,
                    resource: rendering_server::BindingResource::TextureView(&self.view),
                },
                rendering_server::BindGroupEntry {
                    binding: 1,
                    resource: rendering_server::BindingResource::Sampler(&self.sampler),
                },
            ],
            label: p_label,
        });
        bind_group::BindGroupContext {
            bind_group_layout: layout,
            bind_group,
            bind_group_offset: 0,
        }
    }
}
