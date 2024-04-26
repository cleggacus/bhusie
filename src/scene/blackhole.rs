use cgmath::{Vector3, Zero};

pub struct BlackHole {
    pub position: Vector3<f32>,
    pub accretion_disk_rotation: Vector3<f32>,
    pub accretion_disk_inner: f32,
    pub accretion_disk_outer: f32,
    pub rotation_speed: f32,
    pub relativity_sphere_radius: f32,
}

impl BlackHole {
    pub fn new() -> Self {
        Self {
            position: Vector3::zero(),
            accretion_disk_rotation: Vector3::new(0.0, 0.0, 0.0),
            accretion_disk_inner: 1.9,
            accretion_disk_outer: 6.0,
            rotation_speed: 4.0,
            relativity_sphere_radius: 12.0,
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
    pad1: u32,
    accretion_disk_rotation: [f32; 3],
    pad2: u32,
}

impl BlackHoleUniform {
    pub fn new() -> Self {
        Self {
            position: [0.0; 3],
            pad1: 0,
            accretion_disk_rotation: [0.0; 3],
            pad2: 0,
            accretion_disk_inner: 0.0,
            accretion_disk_outer: 0.0,
            rotation_speed: 0.0,
            relativity_sphere_radius: 0.0,
        }
    }

    pub fn update(&mut self, black_hole: &BlackHole) {
        self.position = black_hole.position.into();
        self.accretion_disk_rotation = black_hole.accretion_disk_rotation.into();
        self.accretion_disk_inner = black_hole.accretion_disk_inner;
        self.accretion_disk_outer = black_hole.accretion_disk_outer;
        self.rotation_speed = black_hole.rotation_speed;
        self.relativity_sphere_radius = black_hole.relativity_sphere_radius;
    }
}

impl Default for BlackHoleUniform { fn default() -> Self {
        Self::new()
    }
}

