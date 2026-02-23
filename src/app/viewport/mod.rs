use crate::adapter::renderer::rendering_server::{SubmitToQueue, SubmitToRenderPass};
use crate::adapter::renderer::*;
use crate::app::viewport::grid_texture_atlas::GridTextureAtlasBuilderParameters;

pub mod camera;
pub mod grid_texture_atlas;
pub mod quad;

#[derive(getset::Getters, getset::MutGetters)]
pub struct Viewport {
    quad_render_pipeline_context: quad::QuadRenderPipelineContext,
    camera_context: camera::CameraContext,
    text_brush_context: text::TextBrushContext,

    #[getset(get = "pub", get_mut = "pub")]
    rendering_server: rendering_server::RenderingServer,
}

#[derive(partially::Partial)]
#[partially(derive(Default))]
pub struct ViewportCameraProperties {
    pub origin: glam::Vec3,
    pub direction: glam::Vec3,
    pub fov: f32,
    pub z_near: f32,
    pub z_far: f32,
}

pub struct ViewportRenderParameters<'a, S, I>
where
    S: Into<std::borrow::Cow<'a, text::TextSection<'a>>>,
    I: IntoIterator<Item = S>,
{
    pub quad_instances: &'a [quad::QuadInstance],
    pub text_sections: Option<I>,
}

impl Default for ViewportCameraProperties {
    fn default() -> Self {
        Self {
            origin: glam::Vec3::new(0.0, 0.0, 5.0),
            direction: glam::Vec3::NEG_Z,
            fov: std::f32::consts::PI / 4.0,
            z_near: 0.1,
            z_far: 100.0,
        }
    }
}

impl Viewport {
    pub fn new(
        p_event_loop: &winit::event_loop::ActiveEventLoop,
        p_texture_atlas_builder: grid_texture_atlas::GridTextureAtlasBuilder,
    ) -> anyhow::Result<Self> {
        let window_attributes = winit::window::Window::default_attributes();
        let window = std::sync::Arc::new(p_event_loop.create_window(window_attributes).unwrap());
        let window_size = window.inner_size();
        let builder = rendering_server::RenderingServerBuilder::default();
        let rendering_server = pollster::block_on(
            builder.build(rendering_server::RenderingServerBuilderParameters { window }),
        )?;
        let mut camera_context = camera::CameraContext::new(&rendering_server);
        let texture_atlas = p_texture_atlas_builder.build(GridTextureAtlasBuilderParameters {
            server: &rendering_server,
        })?;
        let quad_render_pipeline_context =
            quad::QuadRenderPipelineContext::new(&rendering_server, &camera_context, texture_atlas);
        camera_context.properties.aspect_ratio =
            window_size.width as f32 / window_size.height as f32;
        let text_brush_context = rendering_server.load_default_text_brush()?;

        Ok(Self {
            rendering_server,
            camera_context,
            quad_render_pipeline_context,
            text_brush_context,
        })
    }

    pub fn resize(&mut self, p_width: u32, p_height: u32) {
        self.rendering_server.resize(p_width, p_height);
        self.camera_context.properties.aspect_ratio = p_width as f32 / p_height as f32;
        self.text_brush_context
            .resize(p_width as _, p_height as _, self.rendering_server.queue());
    }

    pub fn camera_properties(&self) -> ViewportCameraProperties {
        ViewportCameraProperties {
            origin: self.camera_context.properties.origin,
            direction: self.camera_context.properties.direction,
            fov: self.camera_context.properties.fov_y,
            z_near: self.camera_context.properties.z_near,
            z_far: self.camera_context.properties.z_far,
        }
    }

    pub fn set_camera_properties(&mut self, p_camera_properties: PartialViewportCameraProperties) {
        self.camera_context.properties.origin = p_camera_properties
            .origin
            .unwrap_or(self.camera_context.properties.origin);
        self.camera_context.properties.direction = p_camera_properties
            .direction
            .unwrap_or(self.camera_context.properties.direction);
        self.camera_context.properties.fov_y = p_camera_properties
            .fov
            .unwrap_or(self.camera_context.properties.fov_y);
        self.camera_context.properties.z_near = p_camera_properties
            .z_near
            .unwrap_or(self.camera_context.properties.z_near);
        self.camera_context.properties.z_far = p_camera_properties
            .z_far
            .unwrap_or(self.camera_context.properties.z_far);
    }

    pub fn render<'a, S, I>(
        &mut self,
        p_parameters: ViewportRenderParameters<'a, S, I>,
    ) -> anyhow::Result<()>
    where
        S: Into<std::borrow::Cow<'a, text::TextSection<'a>>>,
        I: IntoIterator<Item = S>,
    {
        if let Some(sections) = p_parameters.text_sections {
            self.text_brush_context
                .queue(&self.rendering_server, sections)
                .map_err(anyhow::Error::msg)?;
        }

        self.camera_context
            .submit(self.rendering_server.queue_mut());
        let render_pass_builder = rendering_server::TypicalRenderPassBuilder::default();
        self.rendering_server
            .render_with_typical_pass(
                &mut |p_server: &rendering_server::RenderingServer,
                      p_render_pass: &mut rendering_server::RenderPass| {
                    self.text_brush_context.submit(p_render_pass);

                    self.quad_render_pipeline_context.draw(
                        p_server,
                        p_render_pass,
                        &self.camera_context,
                        p_parameters.quad_instances,
                    );
                },
                &render_pass_builder,
            )
            .map_err(anyhow::Error::msg)
    }
}
