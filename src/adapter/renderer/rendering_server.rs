use super::depth_stencil_context;
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

/// Central server for rendering.
#[derive(getset::Getters)]
pub struct RenderingServer {
    #[getset(get = "pub")]
    surface: Surface<'static>,
    #[getset(get = "pub")]
    surface_configuration: SurfaceConfiguration,
    #[getset(get = "pub")]
    queue: Queue,
    #[getset(get = "pub")]
    device: Device,
    #[getset(get = "pub")]
    window: Arc<winit::window::Window>,
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
    pub depth_stencil_context_builder: Option<depth_stencil_context::DepthStencilContextBuilder>,
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
            depth_stencil_context_builder: Some(
                depth_stencil_context::DepthStencilContextBuilder::default(),
            ),
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

        Ok(RenderingServer {
            queue,

            surface,
            surface_configuration,
            device,
            window: p_parameter.window,
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
    }

    pub fn resize_to_window(&mut self) {
        let size = self.window.inner_size();
        self.resize(size.width, size.height);
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
}
