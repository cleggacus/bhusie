use crate::scene::Scene;

pub struct BlackHoleSettings {
    visible: bool,
}

impl BlackHoleSettings {
    pub fn new() -> Self {
        Self {
            visible: false,
        }
    }

    pub fn ui(&mut self, ctx: &egui::Context, scene: &mut Scene) {
        egui::Window::new("Black Hole Settings")
            .open(&mut self.visible)
            .frame(egui::Frame::window(&egui::Style::default()))
            .show(ctx, |ui| {
                egui::Grid::new("black_hole_settings_grid")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Position:");
                        ui.columns(3, |ui| {
                            ui[0].add(egui::DragValue::new(&mut scene.black_hole.position.x).speed(0.01));
                            ui[1].add(egui::DragValue::new(&mut scene.black_hole.position.y).speed(0.01));
                            ui[2].add(egui::DragValue::new(&mut scene.black_hole.position.z).speed(0.01));
                        });
                        ui.end_row(); 

                        ui.label("Disk Rotation:");
                        ui.columns(3, |ui| {
                            ui[0].add(egui::DragValue::new(&mut scene.black_hole.accretion_disk_rotation.x).speed(0.01));
                            ui[1].add(egui::DragValue::new(&mut scene.black_hole.accretion_disk_rotation.y).speed(0.01));
                            ui[2].add(egui::DragValue::new(&mut scene.black_hole.accretion_disk_rotation.z).speed(0.01));
                        });
                        ui.end_row(); 

                        ui.label("Rotation Speed:");
                        ui.add(egui::DragValue::new(&mut scene.black_hole.rotation_speed).clamp_range(0.0..=10.0).speed(0.01));
                        ui.end_row(); 

                        ui.label("Disk Inner Radius:");
                        ui.add(egui::DragValue::new(&mut scene.black_hole.accretion_disk_inner).clamp_range(0.0..=1000.0).speed(0.01));
                        ui.end_row(); 

                        ui.label("Disk Outer Radius:");
                        ui.add(egui::DragValue::new(&mut scene.black_hole.accretion_disk_outer).clamp_range(0.0..=1000.0).speed(0.01));
                        ui.end_row(); 

                        ui.label("Relativity Radius:");
                        ui.add(egui::DragValue::new(&mut scene.black_hole.relativity_sphere_radius).clamp_range(0.0..=1000.0).speed(0.01));
                        ui.end_row(); 

                        ui.label("Show Disk Texture");
                        let mut show_disk_texture = scene.black_hole.show_disk_texture != 0;
                        ui.checkbox(&mut show_disk_texture, "checked");
                        scene.black_hole.show_disk_texture = show_disk_texture as i32; 
                        ui.end_row(); 

                        ui.label("Redshift");
                        let mut show_red_shift = scene.black_hole.show_red_shift != 0;
                        ui.checkbox(&mut show_red_shift, "checked");
                        scene.black_hole.show_red_shift = show_red_shift as i32; 
                        ui.end_row(); 
                    })
            });
    }

    pub fn show(&mut self) {
        self.visible = true;
    }  
}

impl Default for BlackHoleSettings {
    fn default() -> Self {
        Self::new()
    }
}
