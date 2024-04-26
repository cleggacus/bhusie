use std::f32::consts::PI;

use crate::scene::Scene;

pub struct CameraSettings {
    visible: bool,
}

impl CameraSettings {
    pub fn new() -> Self {
        Self {
            visible: false,
        }
    }

    pub fn ui(&mut self, ctx: &egui::Context, scene: &mut Scene) {
        egui::Window::new("Camera Settings")
            .open(&mut self.visible)
            .frame(egui::Frame::window(&egui::Style::default()))
            .show(ctx, |ui| {
                egui::Grid::new("camera_settings_grid")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Position:");
                        ui.columns(3, |ui| {
                            ui[0].add(egui::DragValue::new(&mut scene.camera.position.x).speed(0.01));
                            ui[1].add(egui::DragValue::new(&mut scene.camera.position.y).speed(0.01));
                            ui[2].add(egui::DragValue::new(&mut scene.camera.position.z).speed(0.01));
                        });
                        ui.end_row(); 

                        ui.label("Rotation (forward vector):");
                        ui.columns(3, |ui| {
                            ui[0].add(egui::DragValue::new(&mut scene.camera.forward.x).speed(0.01));
                            ui[1].add(egui::DragValue::new(&mut scene.camera.forward.y).speed(0.01));
                            ui[2].add(egui::DragValue::new(&mut scene.camera.forward.z).speed(0.01));
                        });
                        ui.end_row(); 

                        ui.label("FOV (rads):");
                        ui.add(egui::DragValue::new(&mut scene.camera.fov).clamp_range(0.0..=PI).speed(0.01));
                        ui.end_row(); 

                        ui.label("Move Speed (units/s):");
                        ui.add(egui::DragValue::new(&mut scene.camera_move_speed).clamp_range(0.0..=100.0).speed(0.01));
                        ui.end_row(); 

                        ui.label("Rotate Speed (rads/s):");
                        ui.add(egui::DragValue::new(&mut scene.camera_rotate_speed).clamp_range(0.0..=2.0).speed(0.01));
                        ui.end_row(); 
                    })
            });
    }

    pub fn show(&mut self) {
        self.visible = true;
    }  
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self::new()
    }
}
