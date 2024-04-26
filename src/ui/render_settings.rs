use wgpu::PresentMode;

use crate::renderer::Renderer;

fn present_mode_to_string(mode: PresentMode) -> String {
    match mode {
        PresentMode::AutoVsync => "Auto Vsync",
        PresentMode::AutoNoVsync => "Auto No Vsync",
        PresentMode::Mailbox => "Mailbox",
        PresentMode::Fifo => "Fifo",
        PresentMode::FifoRelaxed => "Fifo Relaxed",
        PresentMode::Immediate => "Immediate",
    }.into()
}

pub struct RendererSettings {
    visible: bool,
}

impl RendererSettings {
    pub fn new() -> Self {
        Self {
            visible: false,
        }
    }

    pub fn ui(&mut self, ctx: &egui::Context, renderer: &mut Renderer) {
        egui::Window::new("Renderer Settings")
            .open(&mut self.visible)
            .frame(egui::Frame::window(&egui::Style::default()))
            .show(ctx, |ui| {
                egui::Grid::new("renderer_settings_grid")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Present Mode:");

                        let selected_text = present_mode_to_string(renderer.present_mode);

                        egui::ComboBox::from_id_source("renderer_present_mode")
                            .selected_text(selected_text)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut renderer.present_mode,
                                    PresentMode::AutoVsync,
                                    present_mode_to_string(PresentMode::AutoVsync)
                                );

                                ui.selectable_value(
                                    &mut renderer.present_mode,
                                    PresentMode::AutoNoVsync,
                                    present_mode_to_string(PresentMode::AutoNoVsync)
                                );

                                ui.selectable_value(
                                    &mut renderer.present_mode,
                                    PresentMode::Fifo,
                                    present_mode_to_string(PresentMode::Fifo)
                                );

                                ui.selectable_value(
                                    &mut renderer.present_mode,
                                    PresentMode::FifoRelaxed,
                                    present_mode_to_string(PresentMode::FifoRelaxed)
                                );

                                ui.selectable_value(
                                    &mut renderer.present_mode,
                                    PresentMode::Immediate,
                                    present_mode_to_string(PresentMode::Immediate)
                                );

                                ui.selectable_value(
                                    &mut renderer.present_mode,
                                    PresentMode::Mailbox,
                                    "Mailbox"
                                );
                            });

                        ui.end_row(); 

                        ui.label("OED Method:");

                        let selected_text = match renderer.ray_details.integration_method {
                            0 => "Euler",
                            1 => "Runge Kutta",
                            _ => ""
                        };

                        egui::ComboBox::from_id_source("renderer_use_rk")
                            .selected_text(selected_text)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut renderer.ray_details.integration_method,
                                    0,
                                    "Euler"
                                );

                                ui.selectable_value(
                                    &mut renderer.ray_details.integration_method,
                                    1,
                                    "Runge Kutta"
                                );
                            });

                        ui.end_row(); 

                        ui.label("Step size");
                        ui.add(egui::DragValue::new(&mut renderer.ray_details.step_size).speed(0.005).clamp_range(0.005..=1.0));
                        ui.end_row(); 

                        ui.label("Max Iterations");
                        ui.add(egui::DragValue::new(&mut renderer.ray_details.max_iterations));
                        ui.end_row(); 

                        ui.label("Show Disk Texture");

                        let selected_text = match renderer.ray_details.show_disk_texture {
                            0 => "Hide",
                            1 => "Show",
                            _ => ""
                        };

                        egui::ComboBox::from_id_source("Show Disk Texture")
                            .selected_text(selected_text)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut renderer.ray_details.show_disk_texture,
                                    0,
                                    "Hide"
                                );

                                ui.selectable_value(
                                    &mut renderer.ray_details.show_disk_texture,
                                    1,
                                    "Show"
                                );
                            });

                        ui.end_row(); 

                        ui.label("Redshift");

                        let selected_text = match renderer.ray_details.show_red_shift {
                            0 => "No Redshift",
                            1 => "Redshift",
                            _ => ""
                        };

                        egui::ComboBox::from_id_source("Redshift")
                            .selected_text(selected_text)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut renderer.ray_details.show_red_shift,
                                    0,
                                    "No Redshift"
                                );

                                ui.selectable_value(
                                    &mut renderer.ray_details.show_red_shift,
                                    1,
                                    "Redshift"
                                );
                            });

                        ui.end_row(); 
                    })
            });
    }

    pub fn show(&mut self) {
        self.visible = true;
    }  
}

impl Default for RendererSettings {
    fn default() -> Self {
        Self::new()
    }
}
