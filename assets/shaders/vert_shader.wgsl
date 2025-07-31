struct Uniforms {
    view_proj: mat4x4<f32>,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) normal: vec3<f32>,
}

struct Instance {
    @location(3) model_matrix_x: vec4<f32>,
    @location(4) model_matrix_y: vec4<f32>,
    @location(5) model_matrix_z: vec4<f32>,
    @location(6) model_matrix_t: vec4<f32>,
    @location(7) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(input: VertexInput, instance: Instance) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_x,
        instance.model_matrix_y,
        instance.model_matrix_z,
        instance.model_matrix_t,
    );

    var output: VertexOutput;
    output.clip_position = uniforms.view_proj * model_matrix * vec4<f32>(input.position, 1.0);
    output.color = instance.color;
    output.world_normal = normalize((model_matrix * vec4<f32>(input.normal, 0.0)).xyz);
    return output;
}
