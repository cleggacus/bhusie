use std::f32::consts::PI;

use wgpu::PresentMode;

use crate::renderer::{pipelines::fxaa_pipline::{EdgeThresholdMax, EdgeThresholdMin}, Renderer};

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

    pub fn oed(ui: &mut egui::Ui, renderer: &mut Renderer) {
        egui::Grid::new("oed_settings_grid")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
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
            });
    }

    pub fn fxaa(ui: &mut egui::Ui, renderer: &mut Renderer) {
        egui::Grid::new("fxaa_settings_grid")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("Edge Threshold Min");

                let options = [
                    EdgeThresholdMin::Low, 
                    EdgeThresholdMin::Medium, 
                    EdgeThresholdMin::High, 
                    EdgeThresholdMin::Ultra, 
                    EdgeThresholdMin::Extreme, 
                ];

                egui::ComboBox::from_id_source("edge_threshold_min")
                    .selected_text(String::from(renderer.fxaa_details.edge_threshold_min))
                    .show_ui(ui, |ui| {
                        for option in options {
                            ui.selectable_value(
                                &mut renderer.fxaa_details.edge_threshold_min,
                                option,
                                String::from(option)
                            );
                        }
                    });

                ui.end_row(); 

                ui.label("Edge Threshold Max");

                let options = [
                    EdgeThresholdMax::Low, 
                    EdgeThresholdMax::Medium, 
                    EdgeThresholdMax::High, 
                    EdgeThresholdMax::Ultra, 
                    EdgeThresholdMax::Extreme, 
                ];

                egui::ComboBox::from_id_source("edge_threshold_max")
                    .selected_text(String::from(renderer.fxaa_details.edge_threshold_max))
                    .show_ui(ui, |ui| {
                        for option in options {
                            ui.selectable_value(
                                &mut renderer.fxaa_details.edge_threshold_max,
                                option,
                                String::from(option)
                            );
                        }
                    });

                ui.end_row(); 
            });
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

                        let options = [
                            PresentMode::AutoVsync, 
                            PresentMode::AutoNoVsync, 
                            PresentMode::Mailbox,
                            PresentMode::Fifo,
                            PresentMode::FifoRelaxed,
                            PresentMode::Immediate,
                        ];

                        egui::ComboBox::from_id_source("renderer_present_mode")
                            .selected_text(present_mode_to_string(renderer.present_mode))
                            .show_ui(ui, |ui| {
                                for option in options {
                                    ui.selectable_value(
                                        &mut renderer.present_mode,
                                        option,
                                        present_mode_to_string(option)
                                    );
                                }
                            });

                        ui.end_row(); 


                        ui.label("Division Threshold");
                        ui.add(egui::DragValue::new(&mut renderer.ray_details.angle_division_threshold).speed(0.001).clamp_range(0.0..=PI*2.0));
                        ui.end_row(); 

                        ui.label("Highlight Interpolation");
                        let mut highlight_interpolation_bool = renderer.ray_details.highlight_interpolation != 0;
                        ui.checkbox(&mut highlight_interpolation_bool, "checked");
                        renderer.ray_details.highlight_interpolation = highlight_interpolation_bool as i32; 
                        ui.end_row(); 

                        ui.label("Step Mode");
                        ui.checkbox(&mut renderer.step_mode, "checked");
                        ui.end_row(); 

                        if renderer.step_mode {
                            ui.label("Step");
                            if ui.button("Step").clicked() {
                                renderer.step = true;
                            }
                            ui.end_row(); 
                        }
                    });

                    ui.collapsing("OED", |ui| {
                        RendererSettings::oed(ui, renderer);
                    });

                    ui.collapsing("FXAA", |ui| {
                        RendererSettings::fxaa(ui, renderer);
                    });
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
