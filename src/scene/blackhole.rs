use cgmath::{Euler, InnerSpace, Quaternion, Rad, Rotation, Vector3, Zero};

pub struct BlackHole {
    pub position: Vector3<f32>,
    pub accretion_disk_rotation: Vector3<f32>,
    pub accretion_disk_inner: f32,
    pub accretion_disk_outer: f32,
    pub rotation_speed: f32,
    pub relativity_sphere_radius: f32,
    pub show_disk_texture: i32,
    pub show_red_shift: i32,
}

impl BlackHole {
    pub fn new() -> Self {
        Self {
            position: Vector3::zero(),
            accretion_disk_rotation: Vector3::new(0.15, 0.0, 0.2),
            accretion_disk_inner: 2.0,
            accretion_disk_outer: 10.0,
            rotation_speed: 1.0,
            relativity_sphere_radius: 30.0,
            show_disk_texture: 1,
            show_red_shift: 1,
        }
    }
}

impl Default for BlackHole {
    fn default() -> Self {
        Self::new()
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BlackHoleUniform {
    accretion_disk_inner: f32,
    accretion_disk_outer: f32,
    rotation_speed: f32,
    relativity_sphere_radius: f32,
    position: [f32; 3],
    show_disk_texture: i32,
    normal: [f32; 3],
    show_red_shift: i32,
    rotation_matrix: [f32; 12],
    pad: [i32; 9],
}

impl BlackHoleUniform {
    pub fn new() -> Self {
        Self {
            accretion_disk_inner: 0.0,
            accretion_disk_outer: 0.0,
            rotation_speed: 0.0,
            relativity_sphere_radius: 0.0,
            position: [0.0; 3],
            show_disk_texture: 1,
            normal: [0.0; 3],
            show_red_shift: 1,
            rotation_matrix: [0.0; 12],
            pad: [0; 9],
        }
    }

    pub fn update(&mut self, black_hole: &BlackHole) {
        self.position = black_hole.position.into();
        self.accretion_disk_inner = black_hole.accretion_disk_inner;
        self.accretion_disk_outer = black_hole.accretion_disk_outer;
        self.rotation_speed = black_hole.rotation_speed;
        self.relativity_sphere_radius = black_hole.relativity_sphere_radius;
        self.show_disk_texture = black_hole.show_disk_texture;
        self.show_red_shift = black_hole.show_red_shift;

        let q_rotate = Quaternion::from(Euler::new(
            Rad(black_hole.accretion_disk_rotation.x),
            Rad(black_hole.accretion_disk_rotation.y),
            Rad(black_hole.accretion_disk_rotation.z),
        ));

        let up_vector = q_rotate.rotate_vector(Vector3::new(0.0, -1.0, 0.0)).normalize();
        let right_vector = Vector3::new(0.0, 0.0, 1.0).cross(up_vector);
        let forward_vector = right_vector.cross(up_vector);

        self.rotation_matrix = [
            right_vector.x, right_vector.y, right_vector.z, 0.0,
            up_vector.x, up_vector.y, up_vector.z, 0.0,
            forward_vector.x, forward_vector.y, forward_vector.z, 0.0,
        ];

        self.normal = up_vector.into();
    }
}

impl Default for BlackHoleUniform { fn default() -> Self {
        Self::new()
    }
}

