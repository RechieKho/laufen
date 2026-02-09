#![allow(unused_imports)]

use super::texture;
use image::GenericImageView;
use std::num::NonZero;
use std::sync::Arc;
use wgpu::util::DeviceExt;

pub use wgpu::include_wgsl;
pub use wgpu::util::BufferInitDescriptor;
pub use wgpu::BindGroup;
pub use wgpu::BindGroupDescriptor;
pub use wgpu::BindGroupEntry;
pub use wgpu::BindGroupLayout;
pub use wgpu::BindGroupLayoutDescriptor;
pub use wgpu::BindingResource;
pub use wgpu::BindingType;
pub use wgpu::BlendState;
pub use wgpu::Buffer;
pub use wgpu::BufferUsages;
pub use wgpu::Color;
pub use wgpu::ColorTargetState;
pub use wgpu::ColorWrites;
pub use wgpu::CommandEncoder;
pub use wgpu::CompareFunction;
pub use wgpu::DepthBiasState;
pub use wgpu::DepthStencilState;
pub use wgpu::DeviceDescriptor;
pub use wgpu::Face;
pub use wgpu::FrontFace;
pub use wgpu::IndexFormat;
pub use wgpu::InstanceDescriptor;
pub use wgpu::LoadOp;
pub use wgpu::MultisampleState;
pub use wgpu::Operations;
pub use wgpu::PipelineCache;
pub use wgpu::PipelineCompilationOptions;
pub use wgpu::PolygonMode;
pub use wgpu::PowerPreference;
pub use wgpu::PrimitiveState;
pub use wgpu::PrimitiveTopology;
pub use wgpu::QuerySet;
pub use wgpu::RenderPass;
pub use wgpu::RenderPassDepthStencilAttachment;
pub use wgpu::RenderPassDescriptor;
pub use wgpu::RenderPassTimestampWrites;
pub use wgpu::RenderPipeline;
pub use wgpu::Sampler;
pub use wgpu::ShaderModule;
pub use wgpu::ShaderModuleDescriptor;
pub use wgpu::ShaderStages;
pub use wgpu::StencilState;
pub use wgpu::StoreOp;
pub use wgpu::SurfaceError;
pub use wgpu::Texture;
pub use wgpu::TextureFormat;
pub use wgpu::TextureSampleType;
pub use wgpu::TextureUsages;
pub use wgpu::TextureView;
pub use wgpu::TextureViewDimension;
pub use wgpu::VertexBufferLayout;

/// Submit to render pass.
pub trait SubmitToRenderPass {
    fn submit<'a>(&self, p_render_pass: &mut RenderPass<'a>);
}

/// Central server for rendering.
#[allow(unused)]
pub struct RenderingServer {
    instance: wgpu::Instance,
    surface: wgpu::Surface<'static>,
    surface_configuration: wgpu::SurfaceConfiguration,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    window: Arc<winit::window::Window>,

    depth_texture_context: Option<texture::TextureContext>,
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
                backends: wgpu::Backends::PRIMARY,
                ..Default::default()
            },
            adapter_power_preference: PowerPreference::default(),
            adapter_force_fallback_adapter: false,
            device_descriptor: DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: { wgpu::Limits::default() },
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
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

        let instance = wgpu::Instance::new(&self.instance_descriptor);

        let surface = instance.create_surface(p_parameter.window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
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

        let surface_configuration = wgpu::SurfaceConfiguration {
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
            instance,
            surface,
            adapter,
            surface_configuration,
            device,
            queue,
            window: p_parameter.window,

            depth_texture_context: None,
        };

        server.update_depth_texture_context();

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
    pub depth_stencil_default_depth_format: TextureFormat,
    pub depth_stencil_default_depth_write_enabled: bool,
    pub depth_stencil_default_depth_compare: CompareFunction,
    pub depth_stencil_default_stencil: StencilState,
    pub depth_stencil_default_bias: DepthBiasState,
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
            depth_stencil_default_depth_format: RenderingServer::DEPTH_TEXTURE_FORMAT,
            depth_stencil_default_depth_write_enabled: true,
            depth_stencil_default_depth_compare: CompareFunction::Less,
            depth_stencil_default_stencil: StencilState::default(),
            depth_stencil_default_bias: DepthBiasState::default(),
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

impl<'a> TypicalRenderPassBuilder<'a> {
    pub fn build(
        &'a self,
        p_server: &'a RenderingServer,
        p_encoder: &'a mut CommandEncoder,
        p_view: &'a TextureView,
    ) -> RenderPass<'a> {
        p_encoder.begin_render_pass(&RenderPassDescriptor {
            label: self.label,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: p_view,
                resolve_target: self.color_attachment_resolve_target,
                ops: self.color_attachment_operations,
                depth_slice: self.color_attachment_depth_slice,
            })],
            depth_stencil_attachment: p_server.depth_texture_context.as_ref().map(
                |texture_context| RenderPassDepthStencilAttachment {
                    view: &texture_context.view,
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
    const DEPTH_TEXTURE_FORMAT: TextureFormat = TextureFormat::Depth32Float;

    pub fn create_buffer(&self, p_descriptor: &BufferInitDescriptor) -> Buffer {
        self.device.create_buffer_init(p_descriptor)
    }

    pub fn resize(&mut self, p_width: u32, p_height: u32) {
        self.surface_configuration.width = p_width;
        self.surface_configuration.height = p_height;
        self.surface
            .configure(&self.device, &self.surface_configuration);
        self.update_depth_texture_context();
    }

    pub fn resize_to_window(&mut self) {
        let size = self.window.inner_size();
        self.resize(size.width, size.height);
    }

    pub fn load_sample_image(&self) -> anyhow::Result<texture::TextureContext> {
        let default_sample_bytes = include_bytes!("sample_image.png");
        self.load_image_from_bytes(default_sample_bytes, Some("Sample image"))
    }

    pub fn load_image_from_bytes(
        &self,
        p_bytes: &[u8],
        p_label: Option<&str>,
    ) -> anyhow::Result<texture::TextureContext> {
        let image = image::load_from_memory(p_bytes)?;
        self.load_image(&image, p_label)
    }

    pub fn load_image(
        &self,
        p_image: &image::DynamicImage,
        p_label: Option<&str>,
    ) -> anyhow::Result<texture::TextureContext> {
        let rgba = p_image.to_rgba8();
        let dimensions = p_image.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: p_label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        Ok(texture::TextureContext {
            texture,
            view,
            sampler,
        })
    }

    pub fn load_depth(
        &self,
        p_label: Option<&str>,
        p_depth_format: TextureFormat,
    ) -> texture::TextureContext {
        let size = wgpu::Extent3d {
            // 2.
            width: self.surface_configuration.width.max(1),
            height: self.surface_configuration.height.max(1),
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: p_label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: p_depth_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = self.device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        texture::TextureContext {
            texture,
            view,
            sampler,
        }
    }

    fn update_depth_texture_context(&mut self) {
        let depth_texture_context =
            self.load_depth(Some("Depth texture"), Self::DEPTH_TEXTURE_FORMAT);
        self.depth_texture_context = Some(depth_texture_context);
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
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: p_options.pipeline_layout_label,
                    bind_group_layouts: p_parameters.bind_group_layout,
                    immediate_size: p_options.immediate_size,
                });

        let default_color_target_state = &[Some(wgpu::ColorTargetState {
            format: self.surface_configuration.format,
            blend: p_options.fragment_default_color_target_blend,
            write_mask: p_options.fragment_default_color_target_write_mask,
        })];

        self.device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: p_options.pipeline_label,
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: p_parameters.shader_module,
                    entry_point: p_parameters.vertex_entry_point,
                    buffers: p_parameters.vertex_buffer_layouts,
                    compilation_options: p_options.vertex_state_compilation_option.clone(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: p_parameters.shader_module,
                    entry_point: p_parameters.fragment_entry_point,
                    targets: if let Some(targets) = p_parameters.overriding_color_targets {
                        targets
                    } else {
                        default_color_target_state
                    },
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: p_options.primitive,
                depth_stencil: Some(DepthStencilState {
                    format: p_options.depth_stencil_default_depth_format,
                    depth_write_enabled: p_options.depth_stencil_default_depth_write_enabled,
                    depth_compare: p_options.depth_stencil_default_depth_compare,
                    stencil: p_options.depth_stencil_default_stencil.clone(),
                    bias: p_options.depth_stencil_default_bias,
                }),
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
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
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
        R: FnMut(&mut RenderPass),
    {
        self.render(&mut |p_server: &mut RenderingServer,
                          p_encoder: &mut CommandEncoder,
                          p_view: &TextureView| {
            let mut render_pass = p_pass_builder.build(p_server, p_encoder, p_view);
            p_render(&mut render_pass);
        })
    }
}
