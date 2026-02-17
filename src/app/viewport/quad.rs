use super::camera;
use super::grid_texture_atlas;
use crate::adapter::renderer::rendering_server::SubmitToRenderPass;
use crate::adapter::renderer::vertex_buffer::VertexBufferElement;
use crate::adapter::renderer::*;
use repr_trait::C;

pub type QuadTransformationMatrix = glam::Mat4;
pub type RawQuadTransformationMatrix = [[f32; 4]; 4];

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, repr_trait::C)]
pub struct QuadInstance {
    pub raw_transformation_matrix: RawQuadTransformationMatrix,
    pub texture_atlas_index: u32,
}

const QUAD_INSTANCE_BUFFER_ATTRIBUTES: [vertex_buffer::VertexAttribute; 5] = vertex_buffer::vertex_attr_array![5 => Float32x4, 6 => Float32x4, 7 => Float32x4, 8 => Float32x4, 9 => Uint32];

impl buffer::BufferElement for QuadInstance {}

impl vertex_buffer::VertexBufferElement for QuadInstance {
    fn get_vertex_buffer_layout() -> vertex_buffer::VertexBufferLayout<'static> {
        vertex_buffer::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: vertex_buffer::VertexStepMode::Instance,
            attributes: &QUAD_INSTANCE_BUFFER_ATTRIBUTES,
        }
    }
}

impl From<QuadTransformationMatrix> for QuadInstance {
    fn from(p_value: QuadTransformationMatrix) -> Self {
        Self::new(p_value, 0)
    }
}

impl QuadInstance {
    pub fn new(
        p_transformation_matrix: QuadTransformationMatrix,
        p_texture_atlas_index: u32,
    ) -> Self {
        Self {
            texture_atlas_index: p_texture_atlas_index,
            raw_transformation_matrix: bytemuck::cast(p_transformation_matrix),
        }
    }
}

pub type Position = [f32; 3];
pub type TextureCoordinate = [f32; 2];

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, repr_trait::C)]
pub struct QuadVertex {
    pub position: Position,
    pub texture_coordinate: TextureCoordinate,
}
const QUAD_VERTEX_BUFFER_ATTRIBUTES: [vertex_buffer::VertexAttribute; 2] =
    vertex_buffer::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

impl buffer::BufferElement for QuadVertex {}

impl vertex_buffer::VertexBufferElement for QuadVertex {
    fn get_vertex_buffer_layout() -> vertex_buffer::VertexBufferLayout<'static> {
        vertex_buffer::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: vertex_buffer::VertexStepMode::Vertex,
            attributes: &QUAD_VERTEX_BUFFER_ATTRIBUTES,
        }
    }
}

/// A render pipeline context for rendering quads.
pub struct QuadRenderPipelineContext {
    render_pipeline: rendering_server::RenderPipeline,
    render_data: render_data::RenderData,
    texture_atlas: grid_texture_atlas::GridTextureAtlas,
}

impl QuadRenderPipelineContext {
    /// Transformation matrix that transforms the original quad to face upward (+y-axis).
    /// The UV map from -x-axis to +x-axis horizontally, +z-axis to -z-axis vertically.
    pub const QUAD_UPWARD_MATRIX: QuadTransformationMatrix = QuadTransformationMatrix::IDENTITY;

    /// Transformation matrix that transforms the original quad to face downward (-y-axis).
    /// the UV map from +x-axis to -x-axis horizontally, +z-axis to -z-axis vertically.
    pub const QUAD_DOWNWARD_MATRIX: QuadTransformationMatrix = QuadTransformationMatrix::from_cols(
        glam::Vec4::NEG_X,
        glam::Vec4::NEG_Y,
        glam::Vec4::Z,
        glam::Vec4::W,
    );

    /// Transformation matrix that transform model to face left (-x-axis).
    /// the UV map from -y-axis to +y-axis horizontally, +z-axis to -z-axis vertically.
    pub const QUAD_LEFT_MATRIX: QuadTransformationMatrix = QuadTransformationMatrix::from_cols(
        glam::Vec4::Y,
        glam::Vec4::NEG_X,
        glam::Vec4::Z,
        glam::Vec4::W,
    );

    /// Transformation matrix that transform model to face right (+x-axis).
    /// the UV map from +y-axis to -y-axis horizontally, +z-axis to -z-axis vertically.
    pub const QUAD_RIGHT_MATRIX: QuadTransformationMatrix = QuadTransformationMatrix::from_cols(
        glam::Vec4::NEG_Y,
        glam::Vec4::X,
        glam::Vec4::Z,
        glam::Vec4::W,
    );

    /// Transformation matrix that transform model to face backward (-z-axis).
    /// the UV map from -x-axis to +x-axis horizontally, +y-axis to -y-axis vertically.
    pub const QUAD_BACKWARD_MATRIX: QuadTransformationMatrix = QuadTransformationMatrix::from_cols(
        glam::Vec4::X,
        glam::Vec4::Z,
        glam::Vec4::NEG_Y,
        glam::Vec4::W,
    );

    /// Transformation matrix that transform model to face forward (+z-axis).
    /// the UV map from -x-axis to +x-axis horizontally, -y-axis to +y-axis vertically.
    pub const QUAD_FORWARD_MATRIX: QuadTransformationMatrix = QuadTransformationMatrix::from_cols(
        glam::Vec4::X,
        glam::Vec4::NEG_Z,
        glam::Vec4::Y,
        glam::Vec4::W,
    );

    pub const QUAD_SIZE: glam::Vec2 = glam::Vec2::ONE;

    /// A quad that spans 1 by 1 unit and facing upward.
    const QUAD_VERTICES: [QuadVertex; 4] = [
        QuadVertex {
            position: [-0.5, 0.0, 0.5],
            texture_coordinate: [0.0, 1.0],
        },
        QuadVertex {
            position: [0.5, 0.0, 0.5],
            texture_coordinate: [1.0, 1.0],
        },
        QuadVertex {
            position: [0.5, 0.0, -0.5],
            texture_coordinate: [1.0, 0.0],
        },
        QuadVertex {
            position: [-0.5, 0.0, -0.5],
            texture_coordinate: [0.0, 0.0],
        },
    ];
    const QUAD_INDICES: [u16; 6] = [1, 2, 3, 3, 0, 1];

    pub fn new(
        p_server: &rendering_server::RenderingServer,
        p_camera_context: &camera::CameraContext,
        p_texture_atlas: grid_texture_atlas::GridTextureAtlas,
    ) -> Self {
        let mut render_data = render_data::RenderData::default();
        render_data.add_vertex_collections(
            p_server,
            &[vertex_buffer::ToVertexBuffer(&Self::QUAD_VERTICES)],
        );
        render_data.set_indices(p_server, Some(&Self::QUAD_INDICES));
        let shader_module =
            p_server.create_shader_module(rendering_server::include_wgsl!("quad_shader.wgsl"));
        let render_pipeline = p_server.create_pipeline(
            &rendering_server::RenderPipelineParameters {
                shader_module: &shader_module,
                bind_group_layout: &[
                    &p_camera_context.bind_group_context().bind_group_layout,
                    &p_texture_atlas
                        .division_context
                        .bind_group_context()
                        .bind_group_layout,
                    &p_texture_atlas
                        .bounded_texture_context
                        .bind_group_context()
                        .bind_group_layout,
                ],
                vertex_entry_point: Some("vs_main"),
                vertex_buffer_layouts: &[
                    QuadVertex::get_vertex_buffer_layout(),
                    QuadInstance::get_vertex_buffer_layout(),
                ],
                fragment_entry_point: Some("fs_main"),
                overriding_color_targets: None,
            },
            &rendering_server::RenderPipelineOptions::default(),
        );

        Self {
            render_data,
            render_pipeline,
            texture_atlas: p_texture_atlas,
        }
    }

    pub fn draw(
        &self,
        p_server: &rendering_server::RenderingServer,
        p_render_pass: &mut rendering_server::RenderPass,
        p_camera_context: &camera::CameraContext,
        p_instances: &[QuadInstance],
    ) {
        let mut instance_render_data = render_data::RenderData::default();
        instance_render_data
            .add_vertex_collections(p_server, &[vertex_buffer::ToVertexBuffer(p_instances)]);
        instance_render_data.vertex_buffer_slot_offset = 1;

        p_render_pass.set_pipeline(&self.render_pipeline);
        self.render_data.submit(p_render_pass);
        instance_render_data.submit(p_render_pass);
        [
            p_camera_context.bind_group_context(),
            self.texture_atlas.division_context.bind_group_context(),
            self.texture_atlas
                .bounded_texture_context
                .bind_group_context(),
        ]
        .submit(p_render_pass);
        p_render_pass.draw_indexed(
            0..Self::QUAD_INDICES.len() as _,
            0,
            0..p_instances.len() as _,
        );
    }
}
