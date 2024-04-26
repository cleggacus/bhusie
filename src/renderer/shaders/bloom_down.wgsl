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

const INTENSITY: f32 = 2.0;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let screen_size = vec2<f32>(textureDimensions(t_diffuse));
    let src_texel_size = 1.0 / screen_size;

    let x = src_texel_size.x;
    let y = src_texel_size.y;

    // Take 13 samples around current texel:
    // a - b - c
    // - j - k -
    // d - e - f
    // - l - m -
    // g - h - i
    // === ('e' is the current texel) ===
    let a = textureSample(t_diffuse, s_diffuse, vec2<f32>(in.tex_coords.x - 2.0*x,   in.tex_coords.y + 2.0*y)).rgb;
    let b = textureSample(t_diffuse, s_diffuse, vec2<f32>(in.tex_coords.x,           in.tex_coords.y + 2.0*y)).rgb;
    let c = textureSample(t_diffuse, s_diffuse, vec2<f32>(in.tex_coords.x + 2.0*x,   in.tex_coords.y + 2.0*y)).rgb;
    let d = textureSample(t_diffuse, s_diffuse, vec2<f32>(in.tex_coords.x - 2.0*x,   in.tex_coords.y)).rgb;
    let e = textureSample(t_diffuse, s_diffuse, vec2<f32>(in.tex_coords.x,           in.tex_coords.y)).rgb;
    let f = textureSample(t_diffuse, s_diffuse, vec2<f32>(in.tex_coords.x + 2.0*x,   in.tex_coords.y)).rgb;
    let g = textureSample(t_diffuse, s_diffuse, vec2<f32>(in.tex_coords.x - 2.0*x,   in.tex_coords.y - 2.0*y)).rgb;
    let h = textureSample(t_diffuse, s_diffuse, vec2<f32>(in.tex_coords.x,           in.tex_coords.y - 2.0*y)).rgb;
    let i = textureSample(t_diffuse, s_diffuse, vec2<f32>(in.tex_coords.x + 2.0*x,   in.tex_coords.y - 2.0*y)).rgb;
    let j = textureSample(t_diffuse, s_diffuse, vec2<f32>(in.tex_coords.x - x,       in.tex_coords.y + y)).rgb;
    let k = textureSample(t_diffuse, s_diffuse, vec2<f32>(in.tex_coords.x + x,       in.tex_coords.y + y)).rgb;
    let l = textureSample(t_diffuse, s_diffuse, vec2<f32>(in.tex_coords.x - x,       in.tex_coords.y - y)).rgb;
    let m = textureSample(t_diffuse, s_diffuse, vec2<f32>(in.tex_coords.x + x,       in.tex_coords.y - y)).rgb;

    // Apply weighted distribution:
    // 0.5 + 0.125 + 0.125 + 0.125 + 0.125 = 1
    // a,b,d,e * 0.125
    // b,c,e,f * 0.125
    // d,e,g,h * 0.125
    // e,f,h,i * 0.125
    // j,k,l,m * 0.5
    // This shows 5 square areas that are being sampled. But some of them overlap,
    // so to have an energy preserving downsample we need to make some adjustments.
    // The weights are the distributed, so that the sum of j,k,l,m (e.g.)
    // contribute 0.5 to the final color output. The code below is written
    // to effectively yield this sum. We get:
    // 0.125*5 + 0.03125*4 + 0.0625*4 = 1
    var downsample = e*0.125;
    downsample += (a+c+g+i)*0.03125;
    downsample += (b+d+f+h)*0.0625;
    downsample += (j+k+l+m)*0.125;

    // textureSample(t_diffuse, s_diffuse, in.tex_coords)


    // return textureLoad(t_diffuse, vec2<i32>(tex_coords.x, tex_coords.y), 0);
    return vec4<f32>(downsample, 1.0);
}
