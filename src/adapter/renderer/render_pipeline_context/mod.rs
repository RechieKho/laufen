use super::rendering_server;

#[derive(partially::Partial)]
#[partially(derive(Default))]
pub struct RenderPipelineBuilder<'a> {
    pub pipeline_layout_label: Option<&'a str>,
    pub immediate_size: u32,
    pub pipeline_label: Option<&'a str>,
    pub vertex_state_compilation_option: rendering_server::PipelineCompilationOptions<'a>,
    pub fragment_state_compilation_option: rendering_server::PipelineCompilationOptions<'a>,
    pub fragment_default_color_target_blend: Option<rendering_server::BlendState>,
    pub fragment_default_color_target_write_mask: rendering_server::ColorWrites,
    pub primitive: rendering_server::PrimitiveState,
    pub multisample: rendering_server::MultisampleState,
    pub multiview_mask: Option<std::num::NonZeroU32>,
    pub cache: Option<&'a rendering_server::PipelineCache>,
    pub depth_stencil_state: Option<rendering_server::DepthStencilState>,
}

pub struct RenderPipelineBuilderParameters<'a> {
    pub server: &'a rendering_server::RenderingServer,
    pub shader_module_descriptor: rendering_server::ShaderModuleDescriptor<'a>,
    pub bind_group_layout: &'a [&'a rendering_server::BindGroupLayout],
    pub vertex_entry_point: Option<&'a str>,
    pub vertex_buffer_layouts: &'a [rendering_server::VertexBufferLayout<'a>],
    pub fragment_entry_point: Option<&'a str>,
    pub overriding_color_targets: Option<&'a [Option<rendering_server::ColorTargetState>]>,
}

impl<'a> Default for RenderPipelineBuilder<'a> {
    fn default() -> Self {
        Self {
            pipeline_layout_label: Some("Render Pipeline Layout"),
            immediate_size: 0,
            pipeline_label: Some("Render Pipeline"),
            vertex_state_compilation_option: rendering_server::PipelineCompilationOptions::default(
            ),
            fragment_state_compilation_option:
                rendering_server::PipelineCompilationOptions::default(),
            fragment_default_color_target_blend: Some(rendering_server::BlendState::REPLACE),
            fragment_default_color_target_write_mask: rendering_server::ColorWrites::ALL,
            primitive: rendering_server::PrimitiveState {
                topology: rendering_server::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None as Option<rendering_server::IndexFormat>,
                front_face: rendering_server::FrontFace::Ccw,
                cull_mode: Some(rendering_server::Face::Back),
                polygon_mode: rendering_server::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            multisample: rendering_server::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
            depth_stencil_state: None,
        }
    }
}

impl<'a> RenderPipelineBuilder<'a> {
    pub const SAMPLE_SHADER_MODULE_DESCRIPTOR: rendering_server::ShaderModuleDescriptor<'static> =
        rendering_server::include_wgsl!("./sample_shader.wgsl");

    pub fn build(self, p_parameters: RenderPipelineBuilderParameters<'a>) -> RenderPipelineContext {
        let render_pipeline_layout = p_parameters.server.device().create_pipeline_layout(
            &rendering_server::PipelineLayoutDescriptor {
                label: self.pipeline_layout_label,
                bind_group_layouts: p_parameters.bind_group_layout,
                immediate_size: self.immediate_size,
            },
        );

        let default_color_target_state = &[Some(rendering_server::ColorTargetState {
            format: p_parameters.server.surface_configuration().format,
            blend: self.fragment_default_color_target_blend,
            write_mask: self.fragment_default_color_target_write_mask,
        })];

        let shader_module = p_parameters
            .server
            .device()
            .create_shader_module(p_parameters.shader_module_descriptor);

        let pipeline = p_parameters.server.device().create_render_pipeline(
            &rendering_server::RenderPipelineDescriptor {
                label: self.pipeline_label,
                layout: Some(&render_pipeline_layout),
                vertex: rendering_server::VertexState {
                    module: &shader_module,
                    entry_point: p_parameters.vertex_entry_point,
                    buffers: p_parameters.vertex_buffer_layouts,
                    compilation_options: self.vertex_state_compilation_option.clone(),
                },
                fragment: Some(rendering_server::FragmentState {
                    module: &shader_module,
                    entry_point: p_parameters.fragment_entry_point,
                    targets: p_parameters
                        .overriding_color_targets
                        .unwrap_or(default_color_target_state),
                    compilation_options: rendering_server::PipelineCompilationOptions::default(),
                }),
                primitive: self.primitive,
                depth_stencil: self.depth_stencil_state,
                multisample: self.multisample,
                multiview_mask: self.multiview_mask,
                cache: self.cache,
            },
        );

        RenderPipelineContext {
            pipeline,
            shader_module,
        }
    }
}

#[derive(getset::Getters)]
pub struct RenderPipelineContext {
    #[getset(get = "pub")]
    pub(super) pipeline: rendering_server::RenderPipeline,
    #[getset(get = "pub")]
    pub(super) shader_module: rendering_server::ShaderModule,
}
