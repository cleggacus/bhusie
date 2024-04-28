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
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let screen_size = vec2<f32>(textureDimensions(t_diffuse));
    let tex_coord = vec2<i32>(in.tex_coords * screen_size);

    let x = 0.005;
    let y = 0.005;

    // Take 9 samples around current texel:
    // a - b - c
    // d - e - f
    // g - h - i
    // === ('e' is the current texel) ===
    let a = textureSample(t_diffuse, s_diffuse, vec2<f32>(in.tex_coords.x - x, in.tex_coords.y + y)).rgb;
    let b = textureSample(t_diffuse, s_diffuse, vec2<f32>(in.tex_coords.x,     in.tex_coords.y + y)).rgb;
    let c = textureSample(t_diffuse, s_diffuse, vec2<f32>(in.tex_coords.x + x, in.tex_coords.y + y)).rgb;
    let d = textureSample(t_diffuse, s_diffuse, vec2<f32>(in.tex_coords.x - x, in.tex_coords.y)).rgb;
    let e = textureSample(t_diffuse, s_diffuse, vec2<f32>(in.tex_coords.x,     in.tex_coords.y)).rgb;
    let f = textureSample(t_diffuse, s_diffuse, vec2<f32>(in.tex_coords.x + x, in.tex_coords.y)).rgb;
    let g = textureSample(t_diffuse, s_diffuse, vec2<f32>(in.tex_coords.x - x, in.tex_coords.y - y)).rgb;
    let h = textureSample(t_diffuse, s_diffuse, vec2<f32>(in.tex_coords.x,     in.tex_coords.y - y)).rgb;
    let i = textureSample(t_diffuse, s_diffuse, vec2<f32>(in.tex_coords.x + x, in.tex_coords.y - y)).rgb;
    // Apply textureLoad(t_diffuse, vec2<i32>(weighted distribution, by using a 3x3 tent filter:
    //  1   | 1 2 1 |
    // -- * | 2 4 2 |
    // 16   | 1 2 1 |
    var upsample = e*4.0;
    upsample += (b+d+f+h)*2.0;
    upsample += (a+c+g+i);
    upsample *= 1.0 / 16.0;

    return vec4<f32>(upsample, 1.0);
}
