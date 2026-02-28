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
    depth_stencil_context: depth_stencil_context::DepthStencilContext,

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

pub struct ViewportRenderParameters<'a> {
    pub quad_instances: &'a [quad::QuadInstance],
    pub text_sections: &'a [text::TextSection<'a>],
}

impl Default for ViewportCameraProperties {
    fn default() -> Self {
        Self {
            origin: glam::Vec3::new(0.0, 0.0, 5.0),
            direction: glam::Vec3::NEG_Z,
            fov: std::f32::consts::PI / 4.0,
            z_near: 1.0,
            z_far: 25.0,
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
        let depth_stencil_context = depth_stencil_context::DepthStencilContextBuilder::default()
            .build(
                depth_stencil_context::DepthStencilContextBuilderParameters {
                    server: &rendering_server,
                },
            );

        let quad_render_pipeline_context = quad::QuadRenderPipelineContext::new(
            &rendering_server,
            &camera_context,
            texture_atlas,
            &depth_stencil_context,
        );
        camera_context.properties.aspect_ratio =
            window_size.width as f32 / window_size.height as f32;

        let text_brush_context_builder = text::TextBrushContextBuilder {
            depth_stencil_state: Some(depth_stencil_context.depth_stencil_state().clone()),
        };
        let text_brush_context =
            text_brush_context_builder.build_with_default_font(&rendering_server)?;

        Ok(Self {
            rendering_server,
            camera_context,
            quad_render_pipeline_context,
            text_brush_context,
            depth_stencil_context,
        })
    }

    pub fn resize(&mut self, p_width: u32, p_height: u32) {
        self.rendering_server.resize(p_width, p_height);
        self.camera_context.properties.aspect_ratio = p_width as f32 / p_height as f32;
        self.depth_stencil_context
            .rebuild_depth_texture_context(&self.rendering_server);
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

    pub fn render<'a>(&mut self, p_parameters: ViewportRenderParameters<'a>) -> anyhow::Result<()> {
        self.text_brush_context
            .queue(&self.rendering_server, p_parameters.text_sections)
            .map_err(anyhow::Error::msg)?;

        self.camera_context.submit(self.rendering_server.queue());

        self.rendering_server
            .render(&mut |p_server, p_encoder, p_color_texture_view| {
                {
                    let render_pass_builder = render_pass_context::RenderPassContextBuilder {
                        ..Default::default()
                    };
                    let mut pass = render_pass_builder.build(
                        render_pass_context::RenderPassContextBuilderParameters {
                            server: p_server,
                            encoder: p_encoder,
                            color_texture_view: p_color_texture_view,
                            depth_texture_view: self
                                .depth_stencil_context
                                .depth_texture_context()
                                .view(),
                        },
                    );

                    self.quad_render_pipeline_context.draw(
                        p_server,
                        &mut pass.render_pass,
                        &self.camera_context,
                        p_parameters.quad_instances,
                    );
                }

                {
                    let render_pass_builder = render_pass_context::RenderPassContextBuilder {
                        color_attachment_operations: rendering_server::Operations {
                            load: rendering_server::LoadOp::Load,
                            store: rendering_server::StoreOp::Store,
                        },
                        ..Default::default()
                    };
                    let mut pass = render_pass_builder.build(
                        render_pass_context::RenderPassContextBuilderParameters {
                            server: p_server,
                            encoder: p_encoder,
                            color_texture_view: p_color_texture_view,
                            depth_texture_view: self
                                .depth_stencil_context
                                .depth_texture_context()
                                .view(),
                        },
                    );

                    self.text_brush_context.submit(&mut pass.render_pass);
                }
            })
            .map_err(anyhow::Error::msg)
    }
}
