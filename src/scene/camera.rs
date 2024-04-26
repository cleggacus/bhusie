use std::f32::consts::PI;

use cgmath::{InnerSpace, Quaternion, Rad, Rotation, Rotation3, Vector3};

pub struct Camera {
    pub position: Vector3<f32>,
    pub forward: Vector3<f32>,
    pub fov: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            position: Vector3::new(0.0, -1.0, -10.0),
            forward: Vector3::new(0.0, 0.0, 1.0),
            fov: PI/3.0,
        }
    }

    pub fn move_camera(&mut self, amount: Vector3<f32>) {
        self.position += amount;
    }

    pub fn set_position(&mut self, position: Vector3<f32>) {
        self.position = position;
    }

    pub fn rotate_camera(&mut self, yaw: f32, pitch: f32) {
        let right = self.right();

        let q_pitch = Quaternion::from_axis_angle(right, Rad(pitch));
        let q_yaw = Quaternion::from_axis_angle(Vector3::unit_y(), Rad(yaw));

        let q = q_pitch * q_yaw;

        self.forward = q.rotate_vector(self.forward);
    }

    pub fn set_forward(&mut self, forward: Vector3<f32>) {
        self.forward = forward.normalize();
    }

    pub fn position(&self) -> Vector3<f32> {
        self.position
    }

    pub fn forward(&self) -> Vector3<f32> {
        self.forward
    }

    pub fn up(&self) -> Vector3<f32> {
        self.forward.cross(self.right()).normalize()
    }

    pub fn right(&self) -> Vector3<f32> {
        let plane_up = Vector3::<f32>::new(0.0, -1.0, 0.0);
        self.forward.cross(plane_up).normalize()
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}


#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    position: [f32; 3],
    _padding: u32,
    forward: [f32; 3],
    fov: f32,
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            position: [0.0; 3],
            _padding: 0,
            forward: [0.0; 3],
            fov: 0.0 
        }
    }

    pub fn update(&mut self, camera: &Camera) {
        self.position = camera.position.into();
        self.forward = camera.forward.into();
        self.fov = camera.fov;
    }
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self::new()
    }
}

