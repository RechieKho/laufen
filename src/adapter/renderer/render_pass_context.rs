use super::rendering_server;

#[derive(partially::Partial)]
#[partially(derive(Default))]
pub struct RenderPassContextBuilder<'a> {
    pub label: Option<&'a str>,
    pub color_attachment_resolve_target: Option<&'a rendering_server::TextureView>,
    pub color_attachment_operations: rendering_server::Operations<rendering_server::Color>,
    pub color_attachment_depth_slice: Option<u32>,
    pub depth_operations: Option<rendering_server::Operations<f32>>,
    pub stencil_operations: Option<rendering_server::Operations<u32>>,
    pub occlusion_query_set: Option<&'a rendering_server::QuerySet>,
    pub timestamp_writes: Option<rendering_server::RenderPassTimestampWrites<'a>>,
    pub multiview_mask: Option<std::num::NonZeroU32>,
}

impl<'a> Default for RenderPassContextBuilder<'a> {
    fn default() -> Self {
        Self {
            label: Some("Render Pass"),
            color_attachment_resolve_target: None,
            color_attachment_operations: rendering_server::Operations {
                load: rendering_server::LoadOp::Clear(rendering_server::Color {
                    r: 0.1,
                    g: 0.2,
                    b: 0.3,
                    a: 1.0,
                }),
                store: rendering_server::StoreOp::Store,
            },
            color_attachment_depth_slice: None,
            depth_operations: Some(rendering_server::Operations {
                load: rendering_server::LoadOp::Clear(1.0),
                store: rendering_server::StoreOp::Store,
            }),

            stencil_operations: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        }
    }
}

pub struct RenderPassContextBuilderParameters<'a> {
    pub server: &'a rendering_server::RenderingServer,
    pub encoder: &'a mut rendering_server::CommandEncoder,
    pub color_texture_view: &'a rendering_server::TextureView,
    pub depth_texture_view: &'a rendering_server::TextureView,
}

impl<'a> RenderPassContextBuilder<'a> {
    pub fn build(
        self,
        p_parameters: RenderPassContextBuilderParameters<'a>,
    ) -> RenderPassContext<'a> {
        let render_pass =
            p_parameters
                .encoder
                .begin_render_pass(&rendering_server::RenderPassDescriptor {
                    label: self.label,
                    color_attachments: &[Some(rendering_server::RenderPassColorAttachment {
                        view: p_parameters.color_texture_view,
                        resolve_target: self.color_attachment_resolve_target,
                        ops: self.color_attachment_operations,
                        depth_slice: self.color_attachment_depth_slice,
                    })],
                    depth_stencil_attachment: Some(
                        rendering_server::RenderPassDepthStencilAttachment {
                            view: p_parameters.depth_texture_view,
                            depth_ops: self.depth_operations,
                            stencil_ops: self.stencil_operations,
                        },
                    ),
                    occlusion_query_set: self.occlusion_query_set,
                    timestamp_writes: self.timestamp_writes.clone(),
                    multiview_mask: self.multiview_mask,
                });

        RenderPassContext { render_pass }
    }
}

pub struct RenderPassContext<'a> {
    pub render_pass: rendering_server::RenderPass<'a>,
}
