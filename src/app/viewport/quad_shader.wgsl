struct VertexInput {
  @location(0) position: vec3<f32>,
  @location(1) texture_coordinate: vec2<f32>,
};

struct InstanceInput {
  @location(5) transformation_matrix_a: vec4<f32>,
  @location(6) transformation_matrix_b: vec4<f32>,
  @location(7) transformation_matrix_c: vec4<f32>,
  @location(8) transformation_matrix_d: vec4<f32>,
}

struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) texture_coordinate: vec2<f32>,
};

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
  out.position = instance_transformation * vec4<f32>(model.position.xyz, 1.0);
  out.texture_coordinate = model.texture_coordinate;
  return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  return vec4<f32>(0.7, 0.7, 0.7, 1.0);
}
