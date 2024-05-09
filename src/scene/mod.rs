use winit::keyboard::KeyCode;

use crate::{input_manager::InputManager, renderer::{material::MaterialArrayBuffer, model, triangle::ModelArrayBuffer}, timer::Timer};

use self::{blackhole::BlackHole, camera::Camera};

pub mod camera;
pub mod blackhole;

pub struct Scene {
    pub black_hole: BlackHole,
    pub camera: Camera,
    pub camera_move_speed: f32,
    pub camera_rotate_speed: f32,
    pub materials: MaterialArrayBuffer,
    pub models: ModelArrayBuffer,
}

impl Scene {
    pub fn new() -> Self {
        let mut models = ModelArrayBuffer::new();

        model::load_model("./src/renderer/objects/lucy.obj", 
            "Lucy",
            &mut models,
        );

        Self {
            black_hole: BlackHole::new(),
            camera: Camera::new(),
            camera_move_speed: 7.5,
            camera_rotate_speed: 0.15,
            materials: MaterialArrayBuffer::new(),
            models,
        }
    }

    pub fn update(&mut self, timer: &Timer, input_manager: &InputManager) {
        let dt = timer.delta_time().as_secs_f32();
        let camera = &mut self.camera;

        if input_manager.is_key_down(KeyCode::KeyW) {
            camera.move_camera(self.camera_move_speed * dt * camera.forward());
        } else if input_manager.is_key_down(KeyCode::KeyS) {
            camera.move_camera(self.camera_move_speed * dt * -camera.forward());
        }

        if input_manager.is_key_down(KeyCode::KeyD) {
            camera.move_camera(self.camera_move_speed * dt * camera.right());
        } else if input_manager.is_key_down(KeyCode::KeyA) {
            camera.move_camera(self.camera_move_speed * dt * -camera.right());
        }

        if input_manager.is_key_down(KeyCode::KeyQ) {
            camera.move_camera(self.camera_move_speed * dt * camera.up());
        } else if input_manager.is_key_down(KeyCode::KeyE) {
            camera.move_camera(self.camera_move_speed * dt * -camera.up());
        }

        if input_manager.is_left_mouse_down() {
            let (x, y) = input_manager.mouse_move();
            let yaw = x as f32 * self.camera_rotate_speed * dt;
            let pitch = -y as f32 * self.camera_rotate_speed * dt;
            camera.rotate_camera(yaw, pitch);
        }

        let (x, y) = input_manager.joy_right();
        let yaw = x * self.camera_rotate_speed * dt * 10.0;
        let pitch = y * self.camera_rotate_speed * dt * 10.0;
        camera.rotate_camera(yaw, pitch);

        let (x, y) = input_manager.joy_left();
        camera.move_camera(self.camera_move_speed * dt * camera.forward() * 1.5 * y);
        camera.move_camera(self.camera_move_speed * dt * camera.right() * 1.5 * x);

        let (_, y) = input_manager.dpad();
        camera.move_camera(self.camera_move_speed * dt * camera.up() * y);

        // let time = timer.total_time().as_secs_f32() * 0.25;
        // self.models.get_mut(0).unwrap().position.x = time.sin() * 70.0;
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}
