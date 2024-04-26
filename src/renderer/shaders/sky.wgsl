@group(0) @binding(0) var color_buffer: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(1) var s_sky: sampler;
@group(0) @binding(2) var t_sky: texture_2d<f32>;
@group(0) @binding(3) var t_prev: texture_2d<f32>;

@compute @workgroup_size(8,8,1)
fn main(@builtin(global_invocation_id) GlobalInvocationID: vec3<u32>) {
    let screen_size: vec2<i32> = vec2<i32>(textureDimensions(color_buffer));
    let screen_pos: vec2<i32> = vec2<i32>(i32(GlobalInvocationID.x), i32(GlobalInvocationID.y));

    if screen_pos.x >= screen_size.x || screen_pos.y >= screen_size.y {
        return;
    }

    let p = textureLoad(t_prev, screen_pos, 0);

    if p.a == 0.0 {
        let uv = p.rg;
        let sky_color: vec3<f32> = textureSampleLevel(t_sky, s_sky, uv, 0.0).rgb;
        let miss_color = pow(sky_color, vec3<f32>(5.0));
        textureStore(color_buffer, screen_pos, vec4<f32>(miss_color, 1.0));
    } else {
        textureStore(color_buffer, screen_pos, p);
    }
}

