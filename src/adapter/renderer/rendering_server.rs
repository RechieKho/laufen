use super::text;
use super::texture;
use std::num::NonZero;
use std::sync::Arc;
use wgpu::util::DeviceExt;

pub use wgpu::*;

/// Submit to render pass.
pub trait SubmitToRenderPass {
    fn submit<'a>(&self, p_render_pass: &mut RenderPass<'a>);
}

/// Submit to queue.
pub trait SubmitToQueue {
    fn submit(&self, p_queue: &Queue);
}

pub struct DepthStencilContext {
    pub depth_texture_context: texture::TextureContext,
    pub depth_stencil_state: DepthStencilState,
}

/// Central server for rendering.
#[derive(getset::Getters, getset::MutGetters)]
pub struct RenderingServer {
    #[getset(get = "pub")]
    surface: Surface<'static>,

    #[getset(get = "pub")]
    surface_configuration: SurfaceConfiguration,

    #[getset(get = "pub", get_mut = "pub")]
    queue: Queue,

    #[getset(get = "pub", get_mut = "pub")]
    device: Device,

    #[getset(get = "pub", get_mut = "pub")]
    window: Arc<winit::window::Window>,

    depth_stencil_context: Option<DepthStencilContext>,
}

/// Builder for rendering server with overridable default options.
#[derive(partially::Partial)]
#[partially(derive(Default))]
pub struct RenderingServerBuilder<'a> {
    pub instance_descriptor: InstanceDescriptor,
    pub adapter_power_preference: PowerPreference,
    pub adapter_force_fallback_adapter: bool,
    pub device_descriptor: DeviceDescriptor<'a>,
    pub surface_configuration_usage: TextureUsages,
    pub surface_configuration_view_formats: Vec<TextureFormat>,
    pub surface_configuration_desired_maximum_frame_latency: u32,
}

pub struct RenderingServerBuilderParameters {
    pub window: Arc<winit::window::Window>,
}

impl<'a> Default for RenderingServerBuilder<'a> {
    fn default() -> Self {
        Self {
            instance_descriptor: InstanceDescriptor {
                backends: Backends::PRIMARY,
                ..Default::default()
            },
            adapter_power_preference: PowerPreference::default(),
            adapter_force_fallback_adapter: false,
            device_descriptor: DeviceDescriptor {
                label: None,
                required_features: Features::empty(),
                experimental_features: ExperimentalFeatures::disabled(),
                required_limits: { Limits::default() },
                memory_hints: Default::default(),
                trace: Trace::Off,
            },
            surface_configuration_usage: TextureUsages::RENDER_ATTACHMENT,
            surface_configuration_view_formats: vec![],
            surface_configuration_desired_maximum_frame_latency: 2,
        }
    }
}

impl<'a> RenderingServerBuilder<'a> {
    pub async fn build(
        self,
        p_parameter: RenderingServerBuilderParameters,
    ) -> anyhow::Result<RenderingServer> {
        let size = p_parameter.window.inner_size();

        let instance = Instance::new(&self.instance_descriptor);

        let surface = instance.create_surface(p_parameter.window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: self.adapter_power_preference,
                compatible_surface: Some(&surface),
                force_fallback_adapter: self.adapter_force_fallback_adapter,
            })
            .await?;

        let (device, queue) = adapter.request_device(&self.device_descriptor).await?;

        let surface_capabilities = surface.get_capabilities(&adapter);

        let surface_format = surface_capabilities
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_capabilities.formats[0]);

        let surface_configuration = SurfaceConfiguration {
            usage: self.surface_configuration_usage,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: self.surface_configuration_view_formats.clone(),
            desired_maximum_frame_latency: self.surface_configuration_desired_maximum_frame_latency,
        };

        surface.configure(&device, &surface_configuration);

        let mut server = RenderingServer {
            queue,

            surface,
            surface_configuration,
            device,
            window: p_parameter.window,

            depth_stencil_context: None,
        };

        server.update_depth_stencil_context();

        Ok(server)
    }
}

/// Overridable default options for creating render pipeline.
#[derive(partially::Partial)]
#[partially(derive(Default))]
pub struct RenderPipelineOptions<'a> {
    pub pipeline_layout_label: Option<&'a str>,
    pub immediate_size: u32,
    pub pipeline_label: Option<&'a str>,
    pub vertex_state_compilation_option: PipelineCompilationOptions<'a>,
    pub fragment_state_compilation_option: PipelineCompilationOptions<'a>,
    pub fragment_default_color_target_blend: Option<BlendState>,
    pub fragment_default_color_target_write_mask: ColorWrites,
    pub primitive: PrimitiveState,
    pub multisample: MultisampleState,
    pub multiview_mask: Option<NonZero<u32>>,
    pub cache: Option<&'a PipelineCache>,
}

impl<'a> Default for RenderPipelineOptions<'a> {
    fn default() -> Self {
        Self {
            pipeline_layout_label: Some("Render Pipeline Layout"),
            immediate_size: 0,
            pipeline_label: Some("Render Pipeline"),
            vertex_state_compilation_option: PipelineCompilationOptions::default(),
            fragment_state_compilation_option: PipelineCompilationOptions::default(),
            fragment_default_color_target_blend: Some(BlendState::REPLACE),
            fragment_default_color_target_write_mask: ColorWrites::ALL,
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None as Option<IndexFormat>,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        }
    }
}

/// Parameters for creating render pipeline.
pub struct RenderPipelineParameters<'a> {
    pub shader_module: &'a ShaderModule,
    pub bind_group_layout: &'a [&'a BindGroupLayout],
    pub vertex_entry_point: Option<&'a str>,
    pub vertex_buffer_layouts: &'a [VertexBufferLayout<'a>],
    pub fragment_entry_point: Option<&'a str>,
    pub overriding_color_targets: Option<&'a [Option<ColorTargetState>]>,
}

#[derive(partially::Partial)]
#[partially(derive(Default))]
pub struct TypicalRenderPassBuilder<'a> {
    pub label: Option<&'a str>,
    pub color_attachment_resolve_target: Option<&'a TextureView>,
    pub color_attachment_operations: Operations<Color>,
    pub color_attachment_depth_slice: Option<u32>,
    pub depth_operations: Option<Operations<f32>>,
    pub stencil_operations: Option<Operations<u32>>,
    pub occlusion_query_set: Option<&'a QuerySet>,
    pub timestamp_writes: Option<RenderPassTimestampWrites<'a>>,
    pub multiview_mask: Option<NonZero<u32>>,
}

impl<'a> Default for TypicalRenderPassBuilder<'a> {
    fn default() -> Self {
        Self {
            label: Some("Render Pass"),
            color_attachment_resolve_target: None,
            color_attachment_operations: Operations {
                load: LoadOp::Clear(Color {
                    r: 0.1,
                    g: 0.2,
                    b: 0.3,
                    a: 1.0,
                }),
                store: StoreOp::Store,
            },
            color_attachment_depth_slice: None,
            depth_operations: Some(Operations {
                load: LoadOp::Clear(1.0),
                store: StoreOp::Store,
            }),

            stencil_operations: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        }
    }
}

pub struct TypicalRenderPassBuilderParameters<'a> {
    pub server: &'a RenderingServer,
    pub encoder: &'a mut CommandEncoder,
    pub view: &'a TextureView,
}

impl<'a> TypicalRenderPassBuilder<'a> {
    pub fn build(&'a self, p_parameters: TypicalRenderPassBuilderParameters<'a>) -> RenderPass<'a> {
        p_parameters
            .encoder
            .begin_render_pass(&RenderPassDescriptor {
                label: self.label,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: p_parameters.view,
                    resolve_target: self.color_attachment_resolve_target,
                    ops: self.color_attachment_operations,
                    depth_slice: self.color_attachment_depth_slice,
                })],
                depth_stencil_attachment: p_parameters.server.depth_stencil_context.as_ref().map(
                    |context| RenderPassDepthStencilAttachment {
                        view: &context.depth_texture_context.view,
                        depth_ops: self.depth_operations,
                        stencil_ops: self.stencil_operations,
                    },
                ),
                occlusion_query_set: self.occlusion_query_set,
                timestamp_writes: self.timestamp_writes.clone(),
                multiview_mask: self.multiview_mask,
            })
    }
}

impl RenderingServer {
    pub fn create_buffer(&self, p_descriptor: &util::BufferInitDescriptor) -> Buffer {
        self.device.create_buffer_init(p_descriptor)
    }

    pub fn resize(&mut self, p_width: u32, p_height: u32) {
        self.surface_configuration.width = p_width;
        self.surface_configuration.height = p_height;
        self.surface
            .configure(&self.device, &self.surface_configuration);
        self.update_depth_stencil_context();
    }

    pub fn resize_to_window(&mut self) {
        let size = self.window.inner_size();
        self.resize(size.width, size.height);
    }

    pub fn load_default_text_brush(&self) -> anyhow::Result<text::TextBrushContext> {
        let pack = text::font_pack::FontPack::try_load_default()?;
        self.load_text_brush(pack)
    }

    pub fn load_text_brush<T: text::font_pack::Font>(
        &self,
        p_font_pack: text::font_pack::FontPack<T>,
    ) -> anyhow::Result<text::TextBrushContext<T>> {
        let mut builder =
            wgpu_text::BrushBuilder::using_font(p_font_pack.normal).initial_cache_size((512, 512));

        if let Some(context) = self.depth_stencil_context.as_ref() {
            builder = builder.with_depth_stencil(Some(context.depth_stencil_state.clone()));
        }

        let normal = text::FontId::default();

        let italic = p_font_pack.italic.map(|p_font| builder.add_font(p_font));
        let bold = p_font_pack.bold.map(|p_font| builder.add_font(p_font));
        let bold_italic = p_font_pack
            .bold_italic
            .map(|p_font| builder.add_font(p_font));

        let brush = builder.build(
            &self.device,
            self.surface_configuration.width,
            self.surface_configuration.height,
            self.surface_configuration.format,
        );

        Ok(text::TextBrushContext {
            brush,
            normal,
            italic,
            bold,
            bold_italic,
        })
    }

    fn update_depth_stencil_context(&mut self) {
        let depth_texture_context = texture::TextureContextBuilder::spawn_depth_texture(self);
        let depth_stencil_state = DepthStencilState {
            format: texture::TextureContextBuilder::DEFAULT_DEPTH_TEXTURE_FORMAT,
            depth_write_enabled: true,
            depth_compare: CompareFunction::Less,
            stencil: StencilState::default(),
            bias: DepthBiasState::default(),
        };
        self.depth_stencil_context = Some(DepthStencilContext {
            depth_texture_context,
            depth_stencil_state,
        });
    }

    pub fn create_bind_group_layout(
        &self,
        p_descriptor: &BindGroupLayoutDescriptor,
    ) -> BindGroupLayout {
        self.device.create_bind_group_layout(p_descriptor)
    }

    pub fn create_bind_group(&self, p_descriptor: &BindGroupDescriptor) -> BindGroup {
        self.device.create_bind_group(p_descriptor)
    }

    pub fn create_shader_module(&self, p_descriptor: ShaderModuleDescriptor) -> ShaderModule {
        self.device.create_shader_module(p_descriptor)
    }

    pub fn create_sample_shader_module(&self) -> ShaderModule {
        self.create_shader_module(include_wgsl!("sample_shader.wgsl"))
    }

    pub fn create_pipeline<'a>(
        &'a self,
        p_parameters: &'a RenderPipelineParameters<'a>,
        p_options: &'a RenderPipelineOptions<'a>,
    ) -> RenderPipeline {
        let render_pipeline_layout =
            self.device
                .create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: p_options.pipeline_layout_label,
                    bind_group_layouts: p_parameters.bind_group_layout,
                    immediate_size: p_options.immediate_size,
                });

        let default_color_target_state = &[Some(ColorTargetState {
            format: self.surface_configuration.format,
            blend: p_options.fragment_default_color_target_blend,
            write_mask: p_options.fragment_default_color_target_write_mask,
        })];

        self.device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: p_options.pipeline_label,
                layout: Some(&render_pipeline_layout),
                vertex: VertexState {
                    module: p_parameters.shader_module,
                    entry_point: p_parameters.vertex_entry_point,
                    buffers: p_parameters.vertex_buffer_layouts,
                    compilation_options: p_options.vertex_state_compilation_option.clone(),
                },
                fragment: Some(FragmentState {
                    module: p_parameters.shader_module,
                    entry_point: p_parameters.fragment_entry_point,
                    targets: if let Some(targets) = p_parameters.overriding_color_targets {
                        targets
                    } else {
                        default_color_target_state
                    },
                    compilation_options: PipelineCompilationOptions::default(),
                }),
                primitive: p_options.primitive,
                depth_stencil: self
                    .depth_stencil_context
                    .as_ref()
                    .map(|context| context.depth_stencil_state.clone()),
                multisample: p_options.multisample,
                multiview_mask: p_options.multiview_mask,
                cache: p_options.cache,
            })
    }

    pub fn render<R>(&mut self, p_render: &mut R) -> anyhow::Result<(), SurfaceError>
    where
        R: FnMut(&mut RenderingServer, &mut CommandEncoder, &TextureView),
    {
        self.window.request_redraw();

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        p_render(self, &mut encoder, &view);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn render_with_typical_pass<R>(
        &mut self,
        p_render: &mut R,
        p_pass_builder: &TypicalRenderPassBuilder,
    ) -> anyhow::Result<(), SurfaceError>
    where
        R: FnMut(&RenderingServer, &mut RenderPass),
    {
        self.render(&mut |p_server: &mut RenderingServer,
                          p_encoder: &mut CommandEncoder,
                          p_view: &TextureView| {
            let mut render_pass = p_pass_builder.build(TypicalRenderPassBuilderParameters {
                server: p_server,
                encoder: p_encoder,
                view: p_view,
            });
            p_render(p_server, &mut render_pass);
        })
    }
}
