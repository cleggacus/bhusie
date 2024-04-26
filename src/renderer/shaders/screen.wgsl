// Vertex shader

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.tex_coords.y = out.tex_coords.y;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader

@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t_input: texture_2d<f32>;
@group(0) @binding(2) var t_bloom: texture_2d<f32>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return 0.4 * textureSample(t_input, s, in.tex_coords) + 
        0.6 * textureSample(t_bloom, s, in.tex_coords);
}

