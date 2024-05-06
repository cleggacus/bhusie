use sdl2::{controller::GameController, Sdl};
use winit::{dpi::PhysicalSize, event::{Event, WindowEvent}, event_loop::{EventLoop, EventLoopWindowTarget}, window::{Window, WindowBuilder}};

use crate::{input_manager::InputManager, renderer::Renderer, scene::Scene, timer::Timer, ui::UI};

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    // initialize logger
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Could't initialize logger");
        } else {
            std::env::set_var("RUST_LOG", "info");
            env_logger::init();
        }
    }

    let event_loop = EventLoop::new().unwrap();

    let window = WindowBuilder::new()
        .with_title("Rays do be going brrrrr")
        .build(&event_loop).unwrap();

    App::new(&window).await
        .run(event_loop);
}

struct App<'a> {
    window: &'a Window,
    timer: Timer,
    renderer: Renderer<'a>,
    input_manager: InputManager,
    ui: UI,
    scene: Scene,
    sdl_context: Sdl,
    controller: Option<GameController>
}

impl<'a> App<'a> {
    pub async fn new(window: &'a Window) -> Self {
        let timer = Timer::new();
        let input_manager = InputManager::new();
        let ui = UI::new(window);
        let scene = Scene::new();
        let renderer = Renderer::new(window, &scene).await;
        let sdl_context = sdl2::init().unwrap();

        Self {
            window,
            timer,
            renderer,
            input_manager,
            ui,
            scene,
            sdl_context,
            controller: None,
        }
    }

    pub fn run(&mut self, event_loop: EventLoop<()>) {
        self.controller = self.setup_controller();

        event_loop.run(move |event, elwt| {
            self.input_manager.pre_update();

            for event in self.sdl_context.event_pump().unwrap().poll_iter() {
                self.input_manager.sdl_update(&event);
            }

            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == self.window.id() => {
                    self.handle_window_event(event, elwt);
                },
                _ => {}
            }
        }).unwrap();
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent, elwt: &EventLoopWindowTarget<()>) {
        let event_response = self.ui.egui_state_mut().on_window_event(self.window, event);

        if event_response.repaint {
            self.window.request_redraw();
        }

        match event {
            WindowEvent::CloseRequested => elwt.exit(),
            WindowEvent::Resized(physical_size) => self.resize(*physical_size),
            WindowEvent::RedrawRequested => self.handle_redraw_requested(elwt),
            WindowEvent::KeyboardInput { .. } |
            WindowEvent::MouseWheel { .. } |
            WindowEvent::MouseInput { .. } |
            WindowEvent::CursorMoved { .. } => 
                self.input_manager.window_update(event, event_response.consumed),
            _ => {}
        }
    }

    fn setup_controller(&mut self) -> Option<GameController> {
        let game_controller_subsystem = self.sdl_context.game_controller();

        if game_controller_subsystem.is_err() {
            return None;
        }

        let game_controller_subsystem = game_controller_subsystem.unwrap();

        let available = game_controller_subsystem
            .num_joysticks();

        if available.is_err() {
            return None;
        }

        let controller = (0..available.unwrap())
            .find_map(|id| {
                if !game_controller_subsystem.is_game_controller(id) {
                    return None;
                }

                game_controller_subsystem.open(id).ok()
            });

        if let Some(controller) = &controller {
            log::info!("Controller mapping: {}", controller.mapping());
        }

        controller
    }

    fn update(&mut self) {
        self.timer.update();
        self.ui.update(self.timer.delta_time(), self.window, &mut self.scene, &mut self.renderer);
        self.scene.update(&self.timer, &self.input_manager);
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.renderer.resize(new_size.width, new_size.height);
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let future = async move {
            self.renderer.render(&mut self.ui, &mut self.scene, self.timer.delta_time().as_secs_f32()).await
        };

        pollster::block_on(future)
    }

    fn handle_redraw_requested(&mut self, elwt: &EventLoopWindowTarget<()>) {
        self.update();

        match self.render() {
            Ok(_) => {},
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) =>
                self.resize(self.window.inner_size()),
            Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
            Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
        }
    }

}
