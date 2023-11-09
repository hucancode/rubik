const MAX_LIGHT = 10;
struct VertexInput {
    @location(0) position: vec4<f32>,
    @location(1) normal: vec4<f32>,
    @location(2) color: vec4<f32>,
};
struct VertexOutput {
    @location(0) color: vec4<f32>,
    @location(1) normal: vec4<f32>,
    @location(2) world_position: vec4<f32>,
    @builtin(position) position: vec4<f32>,
};
struct Light {
    @location(0) position: vec3<f32>,
    @location(1) radius: f32,
    @location(2) color: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> world: mat4x4<f32>;
@group(1) @binding(0)
var<uniform> view_proj: mat4x4<f32>;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var result: VertexOutput;
    result.color = input.color;
    result.world_position = world * input.position;
    result.position = view_proj * result.world_position;
    result.normal = input.normal;
    return result;
}

@group(1) @binding(1)
var<uniform> lights: array<Light, MAX_LIGHT>;

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let light = lights[0];
    let light_pos = vec4(light.position,1.0); 
    let light_dir = normalize(light_pos - vertex.world_position);
    let diffuse_strength = max(dot(vertex.normal, light_dir), 0.0);
    return (light.color*0.2 + vertex.color*0.8) * diffuse_strength;
}