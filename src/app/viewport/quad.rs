use crate::adapter::renderer::{instance, render_data, rendering_server, vertex_buffer};
use repr_trait::C;

pub use instance::TransformationMatrix;

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

impl vertex_buffer::VertexBufferElement for QuadVertex {
    fn get_vertex_buffer_layout() -> vertex_buffer::VertexBufferLayout<'static> {
        vertex_buffer::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: vertex_buffer::VertexStepMode::Vertex,
            attributes: &QUAD_VERTEX_BUFFER_ATTRIBUTES,
        }
    }
}

/// Transformation matrix that transforms the original quad to face upward (+y-axis).
/// The UV map from -x-axis to +x-axis horizontally, +z-axis to -z-axis vertically.
pub const QUAD_UPWARD_MATRIX: TransformationMatrix = TransformationMatrix::IDENTITY;

/// Transformation matrix that transforms the original quad to face downward (-y-axis).
/// the UV map from +x-axis to -x-axis horizontally, +z-axis to -z-axis vertically.
pub const QUAD_DOWNWARD_MATRIX: TransformationMatrix = TransformationMatrix::from_cols(
    glam::Vec4::NEG_X,
    glam::Vec4::NEG_Y,
    glam::Vec4::Z,
    glam::Vec4::W,
);

/// Transformation matrix that transform model to face left (-x-axis).
/// the UV map from -y-axis to +y-axis horizontally, +z-axis to -z-axis vertically.
pub const QUAD_LEFT_MATRIX: TransformationMatrix = TransformationMatrix::from_cols(
    glam::Vec4::Y,
    glam::Vec4::NEG_X,
    glam::Vec4::Z,
    glam::Vec4::W,
);

/// Transformation matrix that transform model to face right (+x-axis).
/// the UV map from +y-axis to -y-axis horizontally, +z-axis to -z-axis vertically.
pub const QUAD_RIGHT_MATRIX: TransformationMatrix = TransformationMatrix::from_cols(
    glam::Vec4::NEG_Y,
    glam::Vec4::X,
    glam::Vec4::Z,
    glam::Vec4::W,
);

/// Transformation matrix that transform model to face forward (+z-axis).
/// the UV map from -x-axis to +x-axis horizontally, -y-axis to +y-axis vertically.
pub const QUAD_FORWARD_MATRIX: TransformationMatrix = TransformationMatrix::from_cols(
    glam::Vec4::X,
    glam::Vec4::Z,
    glam::Vec4::NEG_Y,
    glam::Vec4::W,
);

/// Transformation matrix that transform model to face backward (-z-axis).
/// the UV map from -x-axis to +x-axis horizontally, +y-axis to -y-axis vertically.
pub const QUAD_BACKWARD_MATRIX: TransformationMatrix = TransformationMatrix::from_cols(
    glam::Vec4::X,
    glam::Vec4::NEG_Z,
    glam::Vec4::Y,
    glam::Vec4::W,
);

/// Create model-space Quad to be rendered.
/// The quad spans 1 by 1 unit and facing upward.
pub fn create_model_space_quad_render_data(
    p_server: &rendering_server::RenderingServer,
) -> render_data::RenderData {
    const QUAD_VERTICES: [QuadVertex; 4] = [
        QuadVertex {
            position: [0.5, 0.0, 0.5],
            texture_coordinate: [1.0, 0.0],
        },
        QuadVertex {
            position: [-0.5, 0.0, 0.5],
            texture_coordinate: [0.0, 0.0],
        },
        QuadVertex {
            position: [-0.5, 0.0, -0.5],
            texture_coordinate: [1.0, 0.0],
        },
        QuadVertex {
            position: [0.5, 0.0, -0.5],
            texture_coordinate: [1.0, 1.0],
        },
    ];
    const QUAD_INDICES: [u16; 6] = [1, 2, 3, 3, 0, 1];

    let mut data = render_data::RenderData::default();
    data.add_vertex_collections(p_server, &[&QUAD_VERTICES]);
    data.set_indices(p_server, Some(&QUAD_INDICES));
    data
}
