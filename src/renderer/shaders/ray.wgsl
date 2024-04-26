const MAX_MODEL_VERTICES = 512000;
const MAX_MODELS = 1;
const MAX_MATERIALS = 8;

@group(0) @binding(0) var color_buffer: texture_storage_2d<rgba32float, write>;
@group(0) @binding(1) var<uniform> camera: Camera;
@group(0) @binding(2) var<uniform> details: Details;
@group(0) @binding(3) var<storage, read> materials: array<Material, MAX_MATERIALS>;
@group(0) @binding(4) var<storage, read> models: array<Model, MAX_MODELS>;
@group(0) @binding(5) var<uniform> black_hole: BlackHole;

@group(0) @binding(6) var s_temp: sampler;
@group(0) @binding(7) var t_temp: texture_2d<f32>;
@group(0) @binding(8) var s_disk: sampler;
@group(0) @binding(9) var t_disk: texture_2d<f32>;

@group(0) @binding(10) var t_prev: texture_2d<f32>;

struct Material {
    color: vec4<f32>
};

struct Details {
    material_count: i32,
    model_count: i32,
    time: f32,
    integration_method: i32, // 0: eulers, 1: rk
    step_size: f32,
    max_iterations: i32,
    show_disk_texture: i32,
    show_red_shift: i32,
}

struct Ray {
    position: vec3<f32>,
    direction: vec3<f32>,
};

struct Camera {
    position: vec3<f32>,
    forward: vec3<f32>,
    fov: f32,
};

struct Spherical {
    r: f32,
    theta: f32,
    phi: f32,
}

struct Model  {
    position: vec3<f32>,
    visible: i32,
    rotation: vec3<f32>,
    point_count: i32,
    normal_count: i32,
    triangle_count: i32,
    points: array<vec3<f32>, MAX_MODEL_VERTICES>,
    normals: array<vec3<f32>, MAX_MODEL_VERTICES>,
    triangles: array<TriangleIndices, MAX_MODEL_VERTICES>,
    nodes: array<Node, MAX_MODEL_VERTICES>,
    bvh_lookup: array<i32, MAX_MODEL_VERTICES>,
}

struct TriangleIndices {
    p1: i32,
    p2: i32,
    p3: i32,
    n1: i32,
    n2: i32,
    n3: i32,
}

struct Triangle {
    p1: vec3<f32>,
    p2: vec3<f32>,
    p3: vec3<f32>,
    n1: vec3<f32>,
    n2: vec3<f32>,
    n3: vec3<f32>,
}

struct Node {
    min_corner: vec3<f32>,
    left_child: i32,
    max_corner: vec3<f32>,
    obj_count: i32
}

struct RenderState {
    color: vec3<f32>,
    t: f32,
    normal: vec3<f32>,
    hit: bool,
}

struct Plane {
    position: vec3<f32>,
    rotation: vec3<f32>,
}

struct Torus2D {
    inner_radius: f32,
    outer_radius: f32,
    position: vec3<f32>,
    rotation: vec3<f32>,
}

struct BlackHole {
    inner_radius: f32,
    outer_radius: f32,
    rotation_speed: f32,
    relativity_radius: f32,
    position: vec3<f32>,
    rotation: vec3<f32>,
}

struct Sphere {
    radius: f32,
    position: vec3<f32>,
    color: vec3<f32>
}

const PI: f32 = 3.1415926;

const a_21 = 1.0/5.0;

const a_31 = 3.0/40.0;
const a_32 = 9.0/40.0;

const a_41 = 3.0/10.0;
const a_42 = -9.0/10.0;
const a_43 = 6.0/5.0;

const a_51 = -11.0/54.0;
const a_52 = 5.0/2.0;
const a_53 = -70.0/27.0;
const a_54 = 35.0/27.0;

const a_61 = 1631.0/55296.0;
const a_62 = 175.0/512.0;
const a_63 = 575.0/13824.0;
const a_64 = 44275.0/110592.0;
const a_65 = 253.0/4096.0;

const b_1 = 37.0/378.0;
const b_2 = 0.0;
const b_3 = 250.0/621.0;
const b_4 = 125.0/594.0;
const b_5 = 0.0;
const b_6 = 512.0/1771.0;

const b_a_1 = 2825.0/27648.0;
const b_a_2 = 0.0;
const b_a_3 = 18575.0/48384.0;
const b_a_4 = 13525.0/55296.0;
const b_a_5 = 277.0/14336.0;
const b_a_6 = 1.0/4.0;

@compute @workgroup_size(8,8,1)
fn main(@builtin(global_invocation_id) GlobalInvocationID: vec3<u32>) {
    let screen_size: vec2<i32> = vec2<i32>(textureDimensions(color_buffer));
    let screen_pos: vec2<i32> = vec2<i32>(i32(GlobalInvocationID.x), i32(GlobalInvocationID.y));

    if screen_pos.x >= screen_size.x || screen_pos.y >= screen_size.y {
        return;
    }

    let t_prev_size: vec2<i32> = vec2<i32>(textureDimensions(t_prev));

    if t_prev_size.x == 1 && t_prev_size.y == 1 {
        // base case
        let ray = create_ray(screen_pos, screen_size);
        let color = trace_ray(ray);
        textureStore(color_buffer, screen_pos, color);
    } else {
        // recursive
        let size_ratio = vec2<f32>(t_prev_size) / vec2<f32>(screen_size);
        let prev_pos: vec2<f32> = vec2<f32>(screen_pos) * size_ratio;
        let prev_pos_tl: vec2<f32> = floor(prev_pos);

        if abs(prev_pos_tl.x - prev_pos.x) < 0.0001 && abs(prev_pos_tl.y - prev_pos.y) < 0.0001 {
            let p = textureLoad(t_prev, vec2<i32>(prev_pos_tl), 0);
            textureStore(color_buffer, screen_pos, p);
        } else {
            //let prev_pos_tl1: vec2<f32> = clamp(prev_pos_tl + vec2<f32>(-1.0, 0.0), vec2<f32>(0.0, 0.0), vec2<f32>(t_prev_size));
            //let prev_pos_t1l: vec2<f32> = clamp(prev_pos_tl + vec2<f32>(0.0, -1.0), vec2<f32>(0.0, 0.0), vec2<f32>(t_prev_size));
            //let prev_pos_t1l1: vec2<f32> = clamp(prev_pos_tl + vec2<f32>(-1.0, -1.0), vec2<f32>(0.0, 0.0), vec2<f32>(t_prev_size));

            let prev_pos_bl: vec2<f32> = prev_pos_tl + vec2<f32>(0.0, 1.0);
            //let prev_pos_bl1: vec2<f32> = clamp(prev_pos_bl + vec2<f32>(-1.0, 0.0), vec2<f32>(0.0, 0.0), vec2<f32>(t_prev_size));
            //let prev_pos_b1l: vec2<f32> = clamp(prev_pos_bl + vec2<f32>(0.0, 1.0), vec2<f32>(0.0, 0.0), vec2<f32>(t_prev_size));
            //let prev_pos_b1l1: vec2<f32> = clamp(prev_pos_bl + vec2<f32>(-1.0, 1.0), vec2<f32>(0.0, 0.0), vec2<f32>(t_prev_size));

            let prev_pos_tr: vec2<f32> = prev_pos_tl + vec2<f32>(1.0, 0.0);
            //let prev_pos_tr1: vec2<f32> = clamp(prev_pos_tr + vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 0.0), vec2<f32>(t_prev_size));
            //let prev_pos_t1r: vec2<f32> = clamp(prev_pos_tr + vec2<f32>(0.0, -1.0), vec2<f32>(0.0, 0.0), vec2<f32>(t_prev_size));
            //let prev_pos_t1r1: vec2<f32> = clamp(prev_pos_tr + vec2<f32>(1.0, -1.0), vec2<f32>(0.0, 0.0), vec2<f32>(t_prev_size));

            let prev_pos_br: vec2<f32> = prev_pos_tl + vec2<f32>(1.0, 1.0);
            //let prev_pos_br1: vec2<f32> = clamp(prev_pos_br + vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 0.0), vec2<f32>(t_prev_size));
            //let prev_pos_b1r: vec2<f32> = clamp(prev_pos_br + vec2<f32>(0.0, 1.0), vec2<f32>(0.0, 0.0), vec2<f32>(t_prev_size));
            //let prev_pos_b1r1: vec2<f32> = clamp(prev_pos_br + vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 0.0), vec2<f32>(t_prev_size));

            let color_tl = textureLoad(t_prev, vec2<i32>(prev_pos_tl), 0);
            // let color_t1l = textureLoad(t_prev, vec2<i32>(prev_pos_t1l), 0);
            // let color_tl1 = textureLoad(t_prev, vec2<i32>(prev_pos_tl1), 0);
            // let color_t1l1 = textureLoad(t_prev, vec2<i32>(prev_pos_t1l1), 0);

            let color_bl = textureLoad(t_prev, vec2<i32>(prev_pos_bl), 0);
            //let color_b1l = textureLoad(t_prev, vec2<i32>(prev_pos_b1l), 0);
            //let color_bl1 = textureLoad(t_prev, vec2<i32>(prev_pos_bl1), 0);
            //let color_b1l1 = textureLoad(t_prev, vec2<i32>(prev_pos_b1l1), 0);

            let color_tr = textureLoad(t_prev, vec2<i32>(prev_pos_tr), 0);
            //let color_t1r = textureLoad(t_prev, vec2<i32>(prev_pos_t1r), 0);
            //let color_tr1 = textureLoad(t_prev, vec2<i32>(prev_pos_tr1), 0);
            //let color_t1r1 = textureLoad(t_prev, vec2<i32>(prev_pos_t1r1), 0);

            let color_br = textureLoad(t_prev, vec2<i32>(prev_pos_br), 0);
            //let color_b1r = textureLoad(t_prev, vec2<i32>(prev_pos_b1r), 0);
            //let color_br1 = textureLoad(t_prev, vec2<i32>(prev_pos_br1), 0);
            //let color_b1r1 = textureLoad(t_prev, vec2<i32>(prev_pos_b1r1), 0);

            if(
                color_tl.a == 0.0 && color_tr.a == 0.0 && color_bl.a == 0.0 && color_br.a == 0.0
                // color_t1l.a == 0.0 && color_t1r.a == 0.0 && color_b1l.a == 0.0 && color_b1r.a == 0.0 &&
                // color_tl.a == 0.0 && color_tr1.a == 0.0 && color_bl1.a == 0.0 && color_br1.a == 0.0 &&
                // color_t1l1.a == 0.0 && color_t1r1.a == 0.0 && color_b1l1.a == 0.0 && color_b1r1.a == 0.0
            ) {
                let t = prev_pos - prev_pos_tl;

                // let pl1 = cubic_hermite(
                //     color_t1l1.rg,
                //     color_tl1.rg,
                //     color_bl1.rg,
                //     color_b1l1.rg,
                //     t.y
                // );

                // let pr1 = cubic_hermite(
                //     color_t1r1.rg,
                //     color_tr1.rg,
                //     color_br1.rg,
                //     color_b1r1.rg,
                //     t.y
                // );

                // let pl = cubic_hermite(
                //     color_t1l.rg,
                //     color_tl.rg,
                //     color_bl.rg,
                //     color_b1l.rg,
                //     t.y
                // );

                // let pr = cubic_hermite(
                //     color_t1r.rg,
                //     color_tr.rg,
                //     color_br.rg,
                //     color_b1r.rg,
                //     t.y
                // );

                // let p = cubic_hermite(
                //     pr1,
                //     pr,
                //     pl,
                //     pl1,
                //     t.x
                // );

                let uv_tl = color_tl.rg;
                let uv_tr = color_tr.rg;
                let uv_bl = color_bl.rg;
                let uv_br = color_br.rg;

                let uv_l = uv_tl + (uv_bl - uv_tl) * t.y;
                let uv_r = uv_tr + (uv_br - uv_tr) * t.y;

                let p = uv_l + (uv_r - uv_l) * t.x;

                textureStore(color_buffer, screen_pos, vec4<f32>(p, 0.0, 0.0));
            } else {
                let ray = create_ray(screen_pos, screen_size);
                let color = trace_ray(ray);
                textureStore(color_buffer, screen_pos, color);
            }
        }
    }

}

fn create_ray(screen_pos: vec2<i32>, screen_size: vec2<i32>) -> Ray {
    let sm = min(screen_size.x, screen_size.y);
    let increment: f32 = 1.0 / f32(min(screen_size.x, screen_size.y));
    let pos = 2.0 * (vec2<f32>(screen_pos) - vec2<f32>(screen_size) / 2.0) * increment;


    let plane_up = vec3<f32>(0.0, -1.0, 0.0);
    let right = normalize(cross(camera.forward, plane_up));
    let up = normalize(cross(camera.forward, right));

    let fov_factor = 1.0 / tan(camera.fov / 2.0);

    let ray_dir = normalize(pos.x*right + pos.y*up + camera.forward*fov_factor);
    let ray_pos = camera.position;

    return Ray(ray_pos, ray_dir);
}

fn trace_ray_model(ray: Ray, model_index: i32, t_min: f32, t_max: f32) -> RenderState {
    var closest_render_state: RenderState;
    closest_render_state.t = t_max;

    var node: Node = models[model_index].nodes[0];
    var stack: array<Node, 32>;
    var stack_location: u32 = 0;

    while(true) {
        var obj_count = node.obj_count;
        var contents = node.left_child;

        if obj_count == 0 {
            var child_1: Node = models[model_index].nodes[contents];
            var child_2: Node = models[model_index].nodes[contents + 1];

            var distance_1: f32 = hit_aabb(ray, child_1, models[model_index].position);
            var distance_2: f32 = hit_aabb(ray, child_2, models[model_index].position);

            if (distance_1 > distance_2) {
                var temp_dist: f32 = distance_1;
                distance_1 = distance_2;
                distance_2 = temp_dist;

                var temp_child: Node = child_1;
                child_1 = child_2;
                child_2 = temp_child;
            }

            if (distance_1 > closest_render_state.t) {
                if (stack_location == 0) {
                    break;
                } else {
                    stack_location = stack_location - 1;
                    node = stack[stack_location];
                }
            }
            else {
                node = child_1;
                if (distance_2 < closest_render_state.t) {
                    stack[stack_location] = child_2;
                    stack_location = stack_location + 1;
                }
            }
        } else {
            for(var i = 0; i < obj_count; i++) {
                let index = models[model_index].bvh_lookup[contents + i];
                let ti = models[model_index].triangles[index];

                let triangle = Triangle(
                    models[model_index].points[ti.p1] + models[model_index].position,
                    models[model_index].points[ti.p2] + models[model_index].position,
                    models[model_index].points[ti.p3] + models[model_index].position,
                    models[model_index].normals[ti.n1],
                    models[model_index].normals[ti.n2],
                    models[model_index].normals[ti.n3]
                );

                let render_state = hit_triangle(ray, t_min, t_max, triangle);

                if render_state.hit && render_state.t < closest_render_state.t {
                    closest_render_state = render_state;
                }
            }

            if stack_location == 0 {
                break;
            } else {
                stack_location = stack_location - 1;
                node = stack[stack_location];
            }
        }
    }


    return closest_render_state;
}

fn hit_ray(ray: Ray, t_min: f32, t_max: f32, ray_distance: f32, render_triangles: bool) -> RenderState {
    var closest_render_state: RenderState;
    closest_render_state.t = t_max;

    let render_state = hit_black_hole(ray, black_hole, t_min, t_max, ray_distance);

    if render_state.hit && render_state.t < closest_render_state.t {
        closest_render_state = render_state;

    }

    if render_triangles {
        for(var i = 0; i < details.model_count; i++) {
            if models[i].visible != 0 {
                let render_state = trace_ray_model(ray, i, t_min, t_max);

                if render_state.hit && render_state.t < closest_render_state.t {
                    closest_render_state = render_state;

                    let light = normalize(vec3<f32>(0.2, 0.2, -1.0));
                    let diffuse = dot(closest_render_state.normal, light);
                    closest_render_state.color *= diffuse;
                }
            }
        }
    }

    return closest_render_state;
}

struct RKState {
    h2: f32,
    h: f32,
    e_max: f32,
    ray: Ray,
}

fn f(rayPos: vec3<f32>, h2: f32, dist: f32) -> vec3<f32> {
    return -1.5 * h2 * rayPos / pow(pow(dist, 2.0), 2.5);
}

fn next_ray_rk(rk_state_in: RKState) -> RKState {
    var rk_state = rk_state_in;

    let ray = rk_state.ray;

    let dist = length(ray.position - black_hole.position);

    var k_1 = vec3<f32>(0.0);
    var k_2 = vec3<f32>(0.0);
    var k_3 = vec3<f32>(0.0);
    var k_4 = vec3<f32>(0.0);
    var k_5 = vec3<f32>(0.0);
    var k_6 = vec3<f32>(0.0);

    let dydx = f(ray.position, rk_state.h2, dist);
    let yscal = vec3<f32>(1.0); //abs(ray.position) + abs(dydx * h);
    let eps = 1.0;

    let h2 = rk_state.h2;

    while true {
        let h = rk_state.h;

        k_1 = dydx;
        k_2 = f(ray.position + (a_21*k_1)*h, h2, dist);
        k_3 = f(ray.position + (a_31*k_1 + a_32*k_2)*h, h2, dist);
        k_4 = f(ray.position + (a_41*k_1 + a_42*k_2 + a_43*k_2)*h, h2, dist);
        k_5 = f(ray.position + (a_51*k_1 + a_52*k_2 + a_53*k_3 + a_54*k_4)*h, h2, dist);
        k_6 = f(ray.position + (a_61*k_1 + a_62*k_2 + a_63*k_3 + a_64*k_4 + a_65*k_5)*h, h2, dist);

        let e = h * ((b_1-b_a_1)*k_1 + (b_2-b_a_2)*k_2 + (b_3-b_a_3)*k_3 + (b_4-b_a_4)*k_4 + (b_5-b_a_5)*k_5 + (b_6-b_a_6)*k_6);

        rk_state.e_max = max(max(abs(e.x/yscal.x), abs(e.y/yscal.y)), abs(e.z/yscal.z));
        rk_state.e_max = rk_state.e_max / eps;

        if rk_state.e_max <= 1.0 {
            break;
        } 

        let h_temp = 0.9 * h / pow(rk_state.e_max, 0.25);

        if h >= 0.0 {
            rk_state.h = max(h_temp, h*0.1);
        } else {
            rk_state.h = min(h_temp, h*0.1);
        }
    }

    rk_state.ray.direction += rk_state.h * (b_a_1*k_1 + b_a_2*k_2 + b_a_3*k_3 + b_a_4*k_4 + b_a_5*k_5 + b_a_6*k_6);
    rk_state.ray.position += ray.direction * rk_state.h;

    if rk_state.e_max > 0.00002 {
        rk_state.h *= 0.9 * pow(rk_state.e_max, -0.001);
    } else {
        rk_state.h *= 1.0001;
    }

    return rk_state;
}

fn next_ray_euler(in_ray: Ray, step_size: f32, h2: f32) -> Ray {
    var ray = in_ray;

    let dist = length(ray.position - black_hole.position);

    ray.direction += f(ray.position, h2, dist) * step_size;
    ray.position += ray.direction * step_size;

    return ray;
}

fn trace_ray(ray: Ray) -> vec4<f32> {
    var relativity = false;

    if distance(ray.position, black_hole.position) < black_hole.relativity_radius {
        relativity = true;
    }

    let t_max = 999999.0;
    let t_min = 0.00001;
    let h2 = pow(length(cross(ray.position, ray.direction)), 2.0);

    var curr_ray = ray;
    var prev_ray = ray;

    let relativity_sphere = Sphere(black_hole.relativity_radius, black_hole.position, vec3<f32>(0.0));

    var closest_render_state: RenderState;

    var rk_state = RKState(
        h2,
        details.step_size,
        0.0,
        curr_ray,
    );

    var step_size = details.step_size;

    for(var i = 0; i < details.max_iterations; i++) {
        if relativity {
            prev_ray = curr_ray;

            if details.integration_method == 0 {
                curr_ray = next_ray_euler(curr_ray, details.step_size, h2);
            } else {
                rk_state = next_ray_rk(rk_state);
                curr_ray = rk_state.ray;
                step_size = rk_state.h;
            }

            prev_ray.direction = curr_ray.direction;

            let ray_distance = distance(ray.position, black_hole.position);
            closest_render_state = hit_ray(prev_ray, t_min, step_size, ray_distance, false);

            if distance(curr_ray.position, black_hole.position) > black_hole.relativity_radius {
                relativity = false;
            }
        } else {
            let ray_distance = distance(ray.position, black_hole.position);
            let hit_sphere_state = hit_sphere(curr_ray, relativity_sphere, t_min, t_max);
            let render_state = hit_ray(curr_ray, t_min, t_max, ray_distance, true);

            if hit_sphere_state.hit && hit_sphere_state.t < render_state.t {
                curr_ray.position += curr_ray.direction * hit_sphere_state.t;
                relativity = true;
            } else {
                closest_render_state = render_state;
                break;
            }
        }

        if closest_render_state.hit {
            break;
        }
    }


    if closest_render_state.hit {
        return vec4<f32>(closest_render_state.color, 1.0);
    }

    let theta = atan2(length(curr_ray.direction.xz), curr_ray.direction.y);
    let phi = atan2(curr_ray.direction.z, curr_ray.direction.x);
    let uv = vec2<f32>((phi + PI) / (2.0 * PI), theta / PI);

    return vec4<f32>(uv, 0.0, 0.0);
}

fn rotate_vector_by_euler_angles(input_vector: vec3<f32>, euler_angles: vec3<f32>) -> vec3<f32> {
    let c1 = cos(euler_angles.x);
    let s1 = sin(euler_angles.x);
    let c2 = cos(euler_angles.y);
    let s2 = sin(euler_angles.y);
    let c3 = cos(euler_angles.z);
    let s3 = sin(euler_angles.z);

    // Rotation matrix

    let rotation_matrix = mat3x3<f32>(
        c1 * c3 - s1 * s2 * s3,
        -c2 * s3,
        s1 * c3 + c1 * s2 * s3,
        c1 * s3 + s1 * s2 * c3,
        c2 * c3,
        s1 * s3 - c1 * s2 * c3,
        -s1 * c2,
        s2,
        c1 * c2
    );

    let rotated_vector = input_vector * rotation_matrix;

    return rotated_vector;
}

fn match_up(up_vector: vec3<f32>, point: vec3<f32>) -> vec3<f32> {
    var n_up_vector = normalize(up_vector);
    let right_vector = cross(vec3<f32>(0.0, 0.0, 1.0), n_up_vector);
    let forward_vector = cross(right_vector, n_up_vector);

    let rotation_matrix = mat3x3<f32>(
        right_vector.x, right_vector.y, right_vector.z,
        n_up_vector.x, n_up_vector.y, n_up_vector.z,
        forward_vector.x, forward_vector.y, forward_vector.z
    );
    
    return rotation_matrix * point;
}

fn hit_black_hole(ray: Ray, black_hole: BlackHole, t_min: f32, t_max: f32, total_distance: f32) -> RenderState {
    let torus = Torus2D(
        black_hole.inner_radius,
        black_hole.outer_radius,
        black_hole.position,
        black_hole.rotation,
    );

    let sphere = Sphere(1.0, black_hole.position, vec3<f32>(0.0));

    var render_state = hit_sphere(ray, sphere, t_min, t_max);

    let disk_hit = hit_torus2d(ray, torus, t_min, t_max);

    if disk_hit.hit && disk_hit.t < render_state.t {
        render_state = disk_hit;

        let intersection = ray.position + ray.direction * render_state.t;
        let dist = distance(black_hole.position, intersection);

        let disk_displacement = match_up(render_state.normal, vec3<f32>(torus.outer_radius, 1.0, torus.outer_radius));
        var disk_density = 1.0 - length(intersection / torus.outer_radius);

        disk_density  *= smoothstep(torus.inner_radius, torus.inner_radius + 1.0, dist);
        disk_density  *= inverseSqrt(dist);
        let optical_depth = pow(10.0 * disk_density, 1.5);

        render_state.color = vec3<f32>(optical_depth);

        if details.show_disk_texture != 0 {
            let r = dist / torus.outer_radius;
            let relative_pos = (intersection - torus.position) / torus.outer_radius;
            let rotated_pos = match_up(render_state.normal, relative_pos);
            let angle = atan2(rotated_pos.z, rotated_pos.x);

            var uv = vec2<f32>(sin(angle + details.time*black_hole.rotation_speed) * r, cos(angle + details.time*black_hole.rotation_speed) * r);
            uv = (uv + 1.0) / 2.0;

            let disk_color: vec3<f32> = textureSampleLevel(t_disk, s_disk, uv, 0.0).rgb;

            render_state.color *= disk_color;
        }

        if details.show_red_shift != 0 {
            let shiftVector = 0.6 * cross(normalize(intersection), normalize(vec3<f32>(0.0, -1.0, 0.0)));
            let velocity = dot(ray.direction, shiftVector);
            let doppler_shift = sqrt((1.0 - velocity) / (1.0 + velocity));
            let gravitational_shift = sqrt(
                (1.0 - 2.0 / dist) / 
                (1.0 - 2.0 / total_distance)
            );

            let shift = clamp(pow(gravitational_shift * doppler_shift, 3.0), 0.0, 1.0);

            let shift_color: vec3<f32> = textureSampleLevel(t_temp, s_temp, vec2<f32>(shift, 0.0), 0.0).rgb;

            render_state.color *= shift_color;
        }
    }

    return render_state;
}


fn hit_torus2d(ray: Ray, torus: Torus2D, t_min: f32, t_max: f32) -> RenderState {
    let plane = Plane(torus.position, torus.rotation);
    let render_state = hit_plane(ray, plane, t_min, t_max);

    if render_state.hit {
        let intersection = ray.position + ray.direction * render_state.t;
        let distance_from_center = distance(torus.position, intersection);

        if distance_from_center < torus.inner_radius || distance_from_center > torus.outer_radius {
            var miss: RenderState;
            miss.hit = false;
            miss.t = t_max;

            return miss;
        }
    }

    return render_state;
}

fn hit_plane(ray: Ray, plane: Plane, t_min: f32, t_max: f32) -> RenderState {
    let normal = rotate_vector_by_euler_angles(vec3<f32>(0.0, -1.0, 0.0), plane.rotation);
    let denom = dot(normal, ray.direction);

    var render_state: RenderState;
    render_state.hit = false;
    render_state.t = t_max;

    let distance = plane.position - ray.position;

    let t = dot(distance, normal) / denom; 

    if t < t_max && t > t_min {
        if denom < 0.0 {
            render_state.normal = -normal;
        } else {
            render_state.normal = normal;
        }

        render_state.color = vec3<f32>(1.0);
        render_state.t = t;
        render_state.hit = true;
        return render_state;
    }

    return render_state;
}

fn hit_aabb(ray: Ray, node: Node, offset: vec3<f32>) -> f32 {
    var inverse_dir: vec3<f32> = vec3<f32>(1.0) / ray.direction;

    let min_corner = node.min_corner + offset;
    let max_corner = node.max_corner + offset;

    var t1: vec3<f32> = (min_corner - ray.position) * inverse_dir;
    var t2: vec3<f32> = (max_corner - ray.position) * inverse_dir;

    var t_min: vec3<f32> = min(t1, t2);
    var t_max: vec3<f32> = max(t1, t2);

    var t_min_axis: f32 = max(max(t_min.x, t_min.y), t_min.z);
    var t_max_axis: f32 = min(min(t_max.x, t_max.y), t_max.z);

    if t_min_axis > t_max_axis || t_max_axis < 0 {
        return 999999.0;
    } else {
        return t_min_axis;
    }
}

fn hit_sphere(ray: Ray, sphere: Sphere, t_min: f32, t_max: f32) -> RenderState {
    var render_state: RenderState;
    render_state.hit = false;
    render_state.t = t_max;

    let oc = ray.position - sphere.position;
    let a = dot(ray.direction, ray.direction);
    let b = 2.0 * dot(oc, ray.direction);
    let c = dot(oc, oc) - sphere.radius * sphere.radius;

    let discriminant = b * b - 4.0 * a * c;

    if discriminant > 0.0 {
        let t1 = (-b - sqrt(discriminant)) / (2.0 * a);
        let t2 = (-b + sqrt(discriminant)) / (2.0 * a);

        var t_closest = t_max;

        if t1 > t_min && t1 < t_max {
            t_closest = t1;
        }
        
        if t2 > t_min && t2 < t_max && t2 < t_closest {
            t_closest = t2;
        }

        if t_closest < t_max && t_closest > t_min {
            let intersectionPoint = ray.position + t_closest * ray.direction;
            let normal = normalize(intersectionPoint - sphere.position);

            render_state.color = sphere.color;
            render_state.t = t_closest;
            render_state.normal = normal;
            render_state.hit = true;

            return render_state;
        }
    }

    return render_state;
}

fn hit_triangle(ray: Ray, t_min: f32, t_max:f32, triangle: Triangle) -> RenderState {
    let corner_a = triangle.p1;
    let corner_b = triangle.p2;
    let corner_c = triangle.p3;

    var render_state: RenderState;
    render_state.hit = false;
    render_state.t = t_max;

    let edge_ab: vec3<f32> = corner_b - corner_a;
    let edge_ac: vec3<f32> = corner_c - corner_a;

    var n: vec3<f32> = normalize(cross(edge_ab, edge_ac));
    var ray_dot_tri: f32 = dot(ray.direction, n);

    if (ray_dot_tri > 0.0) {
        ray_dot_tri = ray_dot_tri * -1;
        n = n * -1;
    }

    if (abs(ray_dot_tri) < 0.00001) {
        return render_state;
    }

    var system_matrix: mat3x3<f32> = mat3x3<f32>(
        ray.direction,
        corner_a - corner_b,
        corner_a - corner_c
    );

    let denominator: f32 = determinant(system_matrix);

    if (abs(denominator) < 0.00001) {
        return render_state;
    }

    system_matrix = mat3x3<f32>(
        ray.direction,
        corner_a - ray.position,
        corner_a - corner_c
    );

    let u: f32 = determinant(system_matrix) / denominator;
    
    if (u < 0.0 || u > 1.0) {
        return render_state;
    }

    system_matrix = mat3x3<f32>(
        ray.direction,
        corner_a - corner_b,
        corner_a - ray.position,
    );
    let v: f32 = determinant(system_matrix) / denominator;
    if (v < 0.0 || u + v > 1.0) {
        return render_state;
    }

    system_matrix = mat3x3<f32>(
        corner_a - ray.position,
        corner_a - corner_b,
        corner_a - corner_c
    );
    let t: f32 = determinant(system_matrix) / denominator;

    if (t > t_min && t < t_max) {
        let normal = (1.0 - u - v) * triangle.n1 + u * triangle.n2 + v * triangle.n3;

        let color = -normal * 0.5 + 0.5;

        render_state.normal = n;
        render_state.color = color;
        render_state.t = t;
        render_state.hit = true;
        return render_state;
    }

    return render_state;
}
