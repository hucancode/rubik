struct VertexInput {
    @location(0) position: vec4<f32>,
    @location(1) normal: vec4<f32>,
    @location(2) color: vec4<f32>,
};
struct VertexOutput {
    @location(0) color: vec4<f32>,
    @builtin(position) position: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> world: mat4x4<f32>;
@group(1) @binding(0)
var<uniform> view_proj: mat4x4<f32>;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var result: VertexOutput;
    result.color = input.color;
    result.position = view_proj * world * input.position;
    return result;
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return vertex.color;
}
