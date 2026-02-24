use super::rendering_server;
use super::texture;

#[derive(getset::Getters)]
pub struct DepthStencilContext {
    #[getset(get = "pub")]
    depth_texture_context: texture::TextureContext,
    #[getset(get = "pub")]
    depth_stencil_state: rendering_server::DepthStencilState,

    depth_texture_context_builder: texture::TextureContextBuilder<'static>,
}

impl DepthStencilContext {
    pub fn rebuild_depth_texture_context(&mut self, p_server: &rendering_server::RenderingServer) {
        self.depth_texture_context = self.depth_texture_context_builder.clone().build(
            texture::TextureContextBuilderParameters {
                server: p_server,
                texture_width: std::num::NonZeroU32::try_from(
                    p_server.surface_configuration().width.max(1),
                )
                .expect("Surface width should not be zero."),
                texture_height: std::num::NonZeroU32::try_from(
                    p_server.surface_configuration().height.max(1),
                )
                .expect("Surface height should not be zero."),
            },
        );
    }
}

pub struct DepthStencilContextBuilder {
    pub depth_texture_context_builder: texture::TextureContextBuilder<'static>,
    pub depth_stencil_state: rendering_server::DepthStencilState,
}

impl Default for DepthStencilContextBuilder {
    fn default() -> Self {
        Self {
            depth_texture_context_builder: texture::TextureContextBuilder::as_depth_texture_builder(
            ),
            depth_stencil_state: rendering_server::DepthStencilState {
                format: texture::TextureContextBuilder::DEFAULT_DEPTH_TEXTURE_FORMAT,
                depth_write_enabled: true,
                depth_compare: rendering_server::CompareFunction::Less,
                stencil: rendering_server::StencilState::default(),
                bias: rendering_server::DepthBiasState::default(),
            },
        }
    }
}

pub struct DepthStencilContextBuilderParameters<'a> {
    pub server: &'a rendering_server::RenderingServer,
}

impl DepthStencilContextBuilder {
    pub fn build<'a>(
        self,
        p_parameters: DepthStencilContextBuilderParameters<'a>,
    ) -> DepthStencilContext {
        DepthStencilContext {
            depth_stencil_state: self.depth_stencil_state,
            depth_texture_context: self.depth_texture_context_builder.clone().build(
                texture::TextureContextBuilderParameters {
                    server: p_parameters.server,
                    texture_width: std::num::NonZeroU32::try_from(
                        p_parameters.server.surface_configuration().width.max(1),
                    )
                    .expect("Surface width should not be zero."),
                    texture_height: std::num::NonZeroU32::try_from(
                        p_parameters.server.surface_configuration().height.max(1),
                    )
                    .expect("Surface height should not be zero."),
                },
            ),
            depth_texture_context_builder: self.depth_texture_context_builder,
        }
    }
}
