use crate::app::renderer::{
    bind_group::ToBindGroupContext, rendering_server::SubmitToRenderPass,
    vertex_buffer::VertexBufferElement,
};

pub mod bind_group;
pub mod buffer;
pub mod index_buffer;
pub mod render_data;
pub mod rendering_server;
pub mod texture;
pub mod vertex_buffer;

#[allow(unused)]
pub struct SampleRenderingContext {
    pub data: render_data::RenderData,
    pub texture_context: texture::TextureContext,
    pub texture_bind_group_context: bind_group::BindGroupContext,
    pub shader_module: rendering_server::ShaderModule,
    pub render_pipeline: rendering_server::RenderPipeline,
}

impl SampleRenderingContext {
    const TRIANGLE_VERTICES: [vertex_buffer::SimpleVertex; 3] = [
        vertex_buffer::SimpleVertex {
            position: [0.0, 0.5, 0.0],
            texture_coordiation: [0.0, 0.0],
        },
        vertex_buffer::SimpleVertex {
            position: [-0.5, -0.5, 0.0],
            texture_coordiation: [0.0, 1.0],
        },
        vertex_buffer::SimpleVertex {
            position: [0.5, -0.5, 0.0],
            texture_coordiation: [1.0, 1.0],
        },
    ];

    const TRIANGLE_INDICES: [u16; 3] = [0, 1, 2];

    pub fn new(
        p_server: &rendering_server::RenderingServer,
    ) -> anyhow::Result<SampleRenderingContext> {
        let data = render_data::RenderData::compile(
            p_server,
            &[&Self::TRIANGLE_VERTICES],
            Some(&Self::TRIANGLE_INDICES),
        );
        let texture_context = p_server.load_sample_image()?;
        let texture_bind_group_context =
            texture_context.to_bind_group_context(p_server, Some("Texture bind group"));
        let shader_module = p_server.create_sample_shader_module();
        let render_pipeline = p_server.create_pipeline(
            &rendering_server::RenderPipelineParameters {
                shader_module: &shader_module,
                bind_group_layout: &[&texture_bind_group_context.bind_group_layout],
                vertex_entry_point: Some("vs_main"),
                vertex_buffer_layouts: &[vertex_buffer::SimpleVertex::get_vertex_buffer_layout()],
                fragment_entry_point: Some("fs_main"),
                overriding_color_targets: None,
            },
            &rendering_server::RenderPipelineOptions::default(),
        );

        Ok(SampleRenderingContext {
            data,
            shader_module,
            texture_context,
            texture_bind_group_context,
            render_pipeline,
        })
    }

    pub fn render(
        &self,
        p_server: &mut rendering_server::RenderingServer,
    ) -> anyhow::Result<(), rendering_server::SurfaceError> {
        let render_pass_builder = rendering_server::TypicalRenderPassBuilder::default();
        p_server.render_with_typical_pass(
            &mut |p_render_pass: &mut rendering_server::RenderPass| {
                p_render_pass.set_pipeline(&self.render_pipeline);
                [&self.texture_bind_group_context].submit(p_render_pass);
                self.data.submit(p_render_pass);
                p_render_pass.draw(0..Self::TRIANGLE_VERTICES.len() as _, 0..1);
                p_render_pass.draw_indexed(0..Self::TRIANGLE_INDICES.len() as _, 0, 0..1);
            },
            &render_pass_builder,
        )
    }
}
