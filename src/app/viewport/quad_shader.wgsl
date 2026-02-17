struct VertexInput {
  @location(0) position: vec3<f32>,
  @location(1) texture_coordinate: vec2<f32>,
};

struct InstanceInput {
  @location(5) transformation_matrix_a: vec4<f32>,
  @location(6) transformation_matrix_b: vec4<f32>,
  @location(7) transformation_matrix_c: vec4<f32>,
  @location(8) transformation_matrix_d: vec4<f32>,
  @location(9) atlas_index: u32
}

struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) texture_coordinate: vec2<f32>,
};

struct CameraUniform {
  transformation_matrix: mat4x4<f32>,
};

struct GridTextureDivisionUniform {
  division: u32
};

@group(0) @binding(0)
var<uniform> camera_uniform: CameraUniform;

@group(1) @binding(0)
var<uniform> grid_texture_division_uniform: GridTextureDivisionUniform;

@vertex
fn vs_main(
  model: VertexInput,
  instance: InstanceInput
) -> VertexOutput {
  let instance_transformation = mat4x4<f32>(
    instance.transformation_matrix_a,
    instance.transformation_matrix_b,
    instance.transformation_matrix_c,
    instance.transformation_matrix_d,
  );

  var out: VertexOutput;
  out.position = camera_uniform.transformation_matrix * instance_transformation * vec4<f32>(model.position.xyz, 1.0);

  let total_cells = grid_texture_division_uniform.division * grid_texture_division_uniform.division;
  if total_cells != 0 && instance.atlas_index < total_cells {
    let texture_coordinate_step = 1.0 / f32(grid_texture_division_uniform.division);
    let texture_coordinate_step_x = f32(instance.atlas_index % grid_texture_division_uniform.division) * texture_coordinate_step;
    let texture_coordinate_step_y = f32(instance.atlas_index / grid_texture_division_uniform.division) * texture_coordinate_step;
    out.texture_coordinate = (model.texture_coordinate / f32(grid_texture_division_uniform.division)) + vec2<f32>(
      texture_coordinate_step_x,
      texture_coordinate_step_y
    );
  } else {
    out.texture_coordinate = model.texture_coordinate;
  }

  return out;
}

@group(2) @binding(0)
var grid_texture: texture_2d<f32>;
@group(2) @binding(1)
var grid_texture_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  return textureSample(grid_texture, grid_texture_sampler, in.texture_coordinate);
}
