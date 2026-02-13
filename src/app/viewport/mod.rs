use crate::adapter::renderer::rendering_server::SubmitToQueue;
use crate::adapter::renderer::*;

pub mod camera;
pub mod quad;

pub struct Viewport {
    rendering_server: rendering_server::RenderingServer,
    camera_context: camera::CameraContext,
    quad_render_pipeline_context: quad::QuadRenderPipelineContext,
}

pub struct ViewportRenderParameters<'a> {
    pub quad_instances: &'a [quad::Instance],
}

impl Viewport {
    pub fn new(p_event_loop: &winit::event_loop::ActiveEventLoop) -> anyhow::Result<Self> {
        let window_attributes = winit::window::Window::default_attributes();
        let window = std::sync::Arc::new(p_event_loop.create_window(window_attributes).unwrap());
        let window_size = window.inner_size();
        let builder = rendering_server::RenderingServerBuilder::default();
        let rendering_server = pollster::block_on(
            builder.build(rendering_server::RenderingServerBuilderParameters { window }),
        )?;
        let mut camera_context = camera::CameraContext::new(&rendering_server);
        let quad_render_pipeline_context =
            quad::QuadRenderPipelineContext::new(&rendering_server, &camera_context);
        camera_context.properties.aspect_ratio =
            window_size.width as f32 / window_size.height as f32;

        Ok(Self {
            rendering_server,
            camera_context,
            quad_render_pipeline_context,
        })
    }

    pub fn resize(&mut self, p_width: u32, p_height: u32) {
        self.rendering_server.resize(p_width, p_height);
        self.camera_context.properties.aspect_ratio = p_width as f32 / p_height as f32;
    }

    pub fn render<'a>(
        &mut self,
        p_parameters: ViewportRenderParameters<'a>,
    ) -> anyhow::Result<(), rendering_server::SurfaceError> {
        self.camera_context
            .submit(self.rendering_server.queue_mut());
        let render_pass_builder = rendering_server::TypicalRenderPassBuilder::default();
        self.rendering_server.render_with_typical_pass(
            &mut |p_server: &rendering_server::RenderingServer,
                  p_render_pass: &mut rendering_server::RenderPass| {
                self.quad_render_pipeline_context.draw(
                    p_server,
                    p_render_pass,
                    &self.camera_context,
                    p_parameters.quad_instances,
                );
            },
            &render_pass_builder,
        )
    }
}
