use std::collections::HashSet;

use winit::{event::{DeviceEvent, ElementState, KeyEvent, MouseButton, WindowEvent}, keyboard::{KeyCode, PhysicalKey}};

pub struct InputManager {
    keys_down: HashSet<KeyCode>,
    left_mouse_down: bool,
    mouse_position: (f64, f64),
    mouse_move: (f64, f64),
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            keys_down: HashSet::new(),
            left_mouse_down: false,
            mouse_position: (0.0, 0.0),
            mouse_move: (0.0, 0.0),
        }
    }

    pub fn is_left_mouse_down(&self) -> bool {
        self.left_mouse_down
    }

    pub fn mouse_move(&self) -> (f64, f64) {
        self.mouse_move
    }

    pub fn is_key_down(&self, key: KeyCode) -> bool {
        self.keys_down.contains(&key)
    }

    pub fn device_update(&mut self, device_event: &DeviceEvent) {
        match device_event {
            DeviceEvent::Button { 
                button, 
                state 
            } => {
                println!("Gamepad button {:?} {:?}", button, state);
            },
            DeviceEvent::Motion { 
                axis, 
                value 
            } => {
                println!("Gamepad axis {:?} {:?}", axis, value);
            },
            _ => {}
        }
    }

    pub fn window_update(&mut self, window_event: &WindowEvent, consumed: bool) {
        self.mouse_move = (0.0, 0.0);
        
        if consumed {
            self.left_mouse_down = false;
        }

        match window_event {
            WindowEvent::KeyboardInput { event, .. } => self.update_keyboard(event),
            WindowEvent::CursorMoved { position, .. } => {
                let (x, y) = self.mouse_position;

                self.mouse_move = (
                    position.x - x,
                    position.y - y,
                );

                self.mouse_position = (
                    position.x,
                    position.y,
                );
            },
            WindowEvent::MouseInput { state, button, .. } => {
                if *button == MouseButton::Left {
                    self.left_mouse_down = *state == ElementState::Pressed;
                }
            },
            _ => {}
        }
    }

    pub fn update_keyboard(&mut self, event: &KeyEvent) {
        if let PhysicalKey::Code(code) = event.physical_key {
            if event.state == ElementState::Pressed {
                self.keys_down.insert(code);
            } else {
                self.keys_down.remove(&code);
            }
        }
    }
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}
