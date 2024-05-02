pub mod camera_settings;
pub mod model_settings;
pub mod black_hole_settings;
pub mod render_settings;

use winit::window::{Fullscreen, Window};

use crate::{renderer::Renderer, scene::Scene};

use self::{black_hole_settings::BlackHoleSettings, camera_settings::CameraSettings, model_settings::ModelSettings, render_settings::RendererSettings};

pub struct UI {
    egui_state: egui_winit::State,
    egui_full_output: Option<egui::FullOutput>,
    delta_time: f64,
    deltas: Vec<f64>,
    fps: f64,
    camera_settings: CameraSettings,
    model_settings: ModelSettings,
    black_hole_settings: BlackHoleSettings,
    render_settings: RendererSettings,
}

impl UI {
    pub fn new(window: &Window) -> Self {
        let egui_ctx = egui::Context::default();
        let viewport_id = egui_ctx.viewport_id();

        let egui_state = egui_winit::State::new(
            egui_ctx, 
            viewport_id,
            &window,
            Some(window.scale_factor() as f32),
            None
        );

        let camera_settings = CameraSettings::new();
        let model_settings = ModelSettings::new();
        let black_hole_settings = BlackHoleSettings::new();
        let render_settings = RendererSettings::new();

        Self {
            egui_state,
            egui_full_output: None,
            delta_time: 0.0,
            deltas: vec![],
            fps: 0.0,
            camera_settings,
            model_settings,
            black_hole_settings,
            render_settings,
        }
    }

    pub fn take_full_output(&mut self) -> Option<egui::FullOutput> {
        self.egui_full_output.take()
    }

    pub fn ctx(&self) -> &egui::Context {
        self.egui_state.egui_ctx()
    }

    pub fn egui_state(&self) -> &egui_winit::State {
        &self.egui_state
    }

    pub fn egui_state_mut(&mut self) -> &mut egui_winit::State {
        &mut self.egui_state
    }

    pub fn update(&mut self, delta_time: instant::Duration, window: &Window, scene: &mut Scene, renderer: &mut Renderer) {
        self.delta_time += delta_time.as_secs_f64();

        self.deltas.push(delta_time.as_secs_f64());

        if self.delta_time > 0.25 {
            let sum = self.deltas.iter().sum::<f64>();
            let delta = sum / self.deltas.len() as f64;

            self.fps = 1.0 / delta;
            self.delta_time = 0.0;
            self.deltas.clear();
        }

        let raw_input = self.egui_state.take_egui_input(window);
        let ctx = self.egui_state.egui_ctx();

        let egui_full_output = ctx.run(raw_input, |egui_ctx| {
            self.camera_settings.ui(egui_ctx, scene);
            self.model_settings.ui(egui_ctx, scene);
            self.black_hole_settings.ui(egui_ctx, scene);
            self.render_settings.ui(egui_ctx, renderer);

            egui::TopBottomPanel::top("menu_bar").show(egui_ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("Open").clicked() {
                            log::info!("Open");
                        } else if ui.button("Save").clicked() {
                            log::info!("Save");
                        }
                    });

                    if ui.input(|i| i.key_pressed(egui::Key::F11)) {
                        let is_fullscreen = window.fullscreen().is_some();

                        window.set_fullscreen(
                            if is_fullscreen {
                                None 
                            } else {
                                Some(Fullscreen::Borderless(None))
                            }
                        );
                    }

                    ui.menu_button("Window", |ui| {
                        let is_fullscreen = window.fullscreen().is_some();

                        let fullscreen_text = if is_fullscreen {
                            "Exit Fullscreen"
                        } else {
                            "Enter Fullscreen"
                        };

                        if ui.button(fullscreen_text).clicked() {
                            window.set_fullscreen(
                                if is_fullscreen {
                                    None 
                                } else {
                                    Some(Fullscreen::Borderless(None))
                                }
                            );
                        }
                    });

                    ui.menu_button("View", |ui| {
                        if ui.button("Model Settings").clicked() {
                            self.model_settings.show();
                        } else if ui.button("Camera Settings").clicked() {
                            self.camera_settings.show();
                        } else if ui.button("Black Hole Settings").clicked() {
                            self.black_hole_settings.show();
                        } else if ui.button("Render Settings").clicked() {
                            self.render_settings.show();
                        }
                    });

                    ui.label(format!("fps: {:.2}", self.fps));
                });
            });
        });

        self.egui_full_output = Some(egui_full_output);
    }
}
