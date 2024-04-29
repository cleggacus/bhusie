use cgmath::{vec3, Euler, Quaternion, Rad, Rotation, Vector3, Zero};

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
            accretion_disk_rotation: Vector3::new(0.0, 0.0, 0.0),
            accretion_disk_inner: 2.0,
            accretion_disk_outer: 10.0,
            rotation_speed: 1.0,
            relativity_sphere_radius: 20.0,
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
    accretion_disk_normal: [f32; 3],
    show_red_shift: i32,
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
            accretion_disk_normal: [0.0; 3],
            show_red_shift: 1,
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

        let rotation = Quaternion::from(Euler {
            x: Rad(black_hole.accretion_disk_rotation.x),
            y: Rad(black_hole.accretion_disk_rotation.y),
            z: Rad(black_hole.accretion_disk_rotation.z),
        });

        self.accretion_disk_normal = rotation.rotate_vector(Vector3::new(0.0, -1.0, 0.0)).into();
    }
}

impl Default for BlackHoleUniform { fn default() -> Self {
        Self::new()
    }
}

