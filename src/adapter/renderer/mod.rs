use super::renderer::{rendering_server::SubmitToRenderPass, vertex_buffer::VertexBufferElement};

pub mod bind_group;
pub mod buffer;
pub mod depth_stencil_context;
pub mod index_buffer;
pub mod instance;
pub mod render_data;
pub mod render_pass_context;
pub mod render_pipeline_context;
pub mod rendering_server;
pub mod text;
pub mod texture;
pub mod uniform_buffer;
pub mod vertex_buffer;

pub struct SampleRenderingContext {
    pub data: render_data::RenderData,
    pub bounded_texture_context: texture::BoundedTextureContext,
    pub render_pipeline_context: render_pipeline_context::RenderPipelineContext,
    pub instances: Vec<instance::Instance>,
    pub text_brush_context: text::TextBrushContext,
    pub depth_stencil_context: depth_stencil_context::DepthStencilContext,
}

impl SampleRenderingContext {
    const TRIANGLE_VERTICES: [vertex_buffer::SimpleVertex; 3] = [
        vertex_buffer::SimpleVertex {
            position: [0.0, 0.5, 0.0],
            texture_coordinate: [0.0, 0.0],
        },
        vertex_buffer::SimpleVertex {
            position: [-0.5, -0.5, 0.0],
            texture_coordinate: [0.0, 1.0],
        },
        vertex_buffer::SimpleVertex {
            position: [0.5, -0.5, 0.0],
            texture_coordinate: [1.0, 1.0],
        },
    ];

    const TRIANGLE_INDICES: [u16; 3] = [0, 1, 2];

    pub fn new(
        p_server: &rendering_server::RenderingServer,
    ) -> anyhow::Result<SampleRenderingContext> {
        let instances = vec![
            (instance::TransformationMatrix::from_translation(glam::Vec3::new(-0.5, 0.0, 0.0))
                * instance::TransformationMatrix::from_scale(glam::Vec3::new(0.5, 0.5, 0.5)))
            .into(),
            (instance::TransformationMatrix::from_translation(glam::Vec3::new(0.5, 0.0, 0.0))
                * instance::TransformationMatrix::from_scale(glam::Vec3::new(0.5, 0.5, 0.5)))
            .into(),
        ] as Vec<instance::Instance>;

        let mut data = render_data::RenderData::default();
        data.add_vertex_collections(
            p_server,
            &[vertex_buffer::ToVertexBuffer(&Self::TRIANGLE_VERTICES)],
        );
        data.add_vertex_collections(
            p_server,
            &[vertex_buffer::ToVertexBuffer(instances.as_slice())],
        );
        data.set_indices(p_server, Some(&Self::TRIANGLE_INDICES));

        let texture_context =
            texture::TextureContextBuilder::default().build_from_sample_image(p_server);
        let bounded_texture_context =
            texture::BoundedTextureContext::new(p_server, texture_context);

        let depth_stencil_context = depth_stencil_context::DepthStencilContextBuilder::default()
            .build(
                depth_stencil_context::DepthStencilContextBuilderParameters { server: p_server },
            );

        let render_pipeline_context_builder = render_pipeline_context::RenderPipelineBuilder {
            depth_stencil_state: Some(depth_stencil_context.depth_stencil_state().clone()),
            ..Default::default()
        };

        let render_pipeline_context =
            render_pipeline_context_builder
                .build(render_pipeline_context::RenderPipelineBuilderParameters {
                server: p_server,
                shader_module_descriptor:
                    render_pipeline_context::RenderPipelineBuilder::SAMPLE_SHADER_MODULE_DESCRIPTOR,
                bind_group_layout: &[bounded_texture_context
                    .bind_group_context()
                    .bind_group_layout()],
                vertex_entry_point: Some("vs_main"),
                vertex_buffer_layouts: &[
                    vertex_buffer::SimpleVertex::get_vertex_buffer_layout(),
                    instance::Instance::get_vertex_buffer_layout(),
                ],
                fragment_entry_point: Some("fs_main"),
                overriding_color_targets: None,
            });

        let text_brush_context_builder = text::TextBrushContextBuilder {
            depth_stencil_state: Some(depth_stencil_context.depth_stencil_state().clone()),
        };
        let text_brush_context = text_brush_context_builder.build_with_default_font(p_server)?;

        Ok(SampleRenderingContext {
            data,
            bounded_texture_context,
            render_pipeline_context,
            instances,
            text_brush_context,
            depth_stencil_context,
        })
    }

    pub fn render(
        &mut self,
        p_server: &mut rendering_server::RenderingServer,
    ) -> anyhow::Result<(), rendering_server::SurfaceError> {
        let text = text::Text::new("Hello World")
            .with_scale(24.0)
            .with_color([1.0, 1.0, 1.0, 1.0]);
        let section = text::TextSection::default().add_text(text);
        self.text_brush_context.queue(p_server, [section]).unwrap();

        p_server.render(&mut |p_server, p_encoder, p_color_texture_view| {
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

                pass.render_pass
                    .set_pipeline(self.render_pipeline_context.pipeline());
                [self.bounded_texture_context.bind_group_context()].submit(&mut pass.render_pass);
                self.data.submit(&mut pass.render_pass);
                pass.render_pass.draw_indexed(
                    0..Self::TRIANGLE_INDICES.len() as _,
                    0,
                    0..self.instances.len() as _,
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
    }
}

#[derive(Default)]
pub struct SampleRenderingApp {
    rendering_server: Option<rendering_server::RenderingServer>,
    rendering_context: Option<SampleRenderingContext>,
}

impl winit::application::ApplicationHandler<()> for SampleRenderingApp {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window_attributes = winit::window::Window::default_attributes();
        let window = std::sync::Arc::new(event_loop.create_window(window_attributes).unwrap());
        let builder = rendering_server::RenderingServerBuilder::default();
        self.rendering_server = Some(
            pollster::block_on(
                builder.build(rendering_server::RenderingServerBuilderParameters { window }),
            )
            .unwrap(),
        );
        if let Some(server) = &self.rendering_server {
            self.rendering_context = Some(SampleRenderingContext::new(server).unwrap());
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::CloseRequested => event_loop.exit(),
            winit::event::WindowEvent::Resized(size) => {
                if let Some(server) = &mut self.rendering_server {
                    server.resize(size.width, size.height);
                    if let Some(context) = &mut self.rendering_context {
                        context.text_brush_context.resize(
                            size.width as _,
                            size.height as _,
                            server.queue(),
                        );
                        context
                            .depth_stencil_context
                            .rebuild_depth_texture_context(server);
                        context.text_brush_context.resize(
                            size.width as _,
                            size.height as _,
                            server.queue(),
                        );
                    }
                }
            }
            winit::event::WindowEvent::RedrawRequested => {
                if let (Some(context), Some(server)) =
                    (&mut self.rendering_context, &mut self.rendering_server)
                {
                    match context.render(server) {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            server.resize_to_window();
                        }
                        Err(e) => {
                            log::error!("Unable to render {}", e);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

pub fn run_sample_rendering_app() -> anyhow::Result<()> {
    env_logger::init();
    let event_loop = winit::event_loop::EventLoop::with_user_event().build()?;
    let mut app = SampleRenderingApp::default();
    event_loop.run_app(&mut app)?;
    Ok(())
}
