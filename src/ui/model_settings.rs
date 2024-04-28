use crate::scene::Scene;

pub struct ModelSettings {
    visible: bool,
}

impl ModelSettings {
    pub fn new() -> Self {
        Self {
            visible: false,
        }
    }

    pub fn ui(&mut self, ctx: &egui::Context, scene: &mut Scene) {
        egui::Window::new("Model Settings")
            .open(&mut self.visible)
            .frame(egui::Frame::window(&egui::Style::default()))
            .show(ctx, |ui| {
                egui::Grid::new("model_settings_grid")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {

                        for i in 0..scene.models.size() {
                            let model = scene.models.get_mut(i).unwrap();
                            let mut ident = String::new();

                            for ci in 0..model.ident_size {
                                ident.push(model.ident[ci]);
                            }

                            ui.label("Ident:");
                            ui.label(ident);
                            ui.end_row(); 

                            ui.label("Position:");
                            ui.columns(3, |ui| {
                                ui[0].add(egui::DragValue::new(&mut model.position.x).speed(0.01));
                                ui[1].add(egui::DragValue::new(&mut model.position.y).speed(0.01));
                                ui[2].add(egui::DragValue::new(&mut model.position.z).speed(0.01));
                            });
                            ui.end_row(); 

                            ui.label("Model Visible");
                            let mut visible = model.visible != 0;
                            ui.checkbox(&mut visible, "checked");
                            model.visible = visible as i32; 
                            ui.end_row(); 
                        }

                    })
            });
    }

    pub fn show(&mut self) {
        self.visible = true;
    }  
}

impl Default for ModelSettings {
    fn default() -> Self {
        Self::new()
    }
}
