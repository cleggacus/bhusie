use std::collections::HashSet;
use winit::{event::{ElementState, KeyEvent, MouseButton, WindowEvent}, keyboard::{KeyCode, PhysicalKey}};

pub struct InputManager {
    keys_down: HashSet<KeyCode>,
    left_mouse_down: bool,
    mouse_position: (f64, f64),
    mouse_move: (f64, f64),
    joy_left: (f32, f32),
    joy_right: (f32, f32),
    dpad: (f32, f32),
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            keys_down: HashSet::new(),
            left_mouse_down: false,
            mouse_position: (0.0, 0.0),
            mouse_move: (0.0, 0.0),
            joy_left: (0.0, 0.0),
            joy_right: (0.0, 0.0),
            dpad: (0.0, 0.0),
        }
    }

    pub fn is_left_mouse_down(&self) -> bool {
        self.left_mouse_down
    }

    pub fn joy_right(&self) -> (f32, f32) {
        self.joy_right
    }

    pub fn joy_left(&self) -> (f32, f32) {
        self.joy_left
    }

    pub fn dpad(&self) -> (f32, f32) {
        self.dpad
    }

    pub fn mouse_move(&self) -> (f64, f64) {
        self.mouse_move
    }

    pub fn is_key_down(&self, key: KeyCode) -> bool {
        self.keys_down.contains(&key)
    }

    pub fn pre_update(&mut self) {
        self.mouse_move = (0.0, 0.0);
    }

    pub fn gilrs_update(&mut self, event: &gilrs::EventType) {
        match event {
            gilrs::EventType::AxisChanged(gilrs::Axis::LeftStickX, amount, _) => {
                self.joy_left.0 = *amount;
            },
            gilrs::EventType::AxisChanged(gilrs::Axis::LeftStickY, amount, _) => {
                self.joy_left.1 = *amount;
            },
            gilrs::EventType::AxisChanged(gilrs::Axis::RightStickX, amount, _) => {
                self.joy_right.0 = *amount;
            },
            gilrs::EventType::AxisChanged(gilrs::Axis::RightStickY, amount, _) => {
                self.joy_right.1 = *amount;

            },
            gilrs::EventType::AxisChanged(gilrs::Axis::DPadX, amount, _) => {
                self.dpad.0 = *amount;
            },
            gilrs::EventType::AxisChanged(gilrs::Axis::DPadY, amount, _) => {
                self.dpad.1 = *amount;
            },
            gilrs::EventType::ButtonChanged(gilrs::Button::DPadLeft, amount, _) => {
                self.dpad.0 = -1.0 * amount;
            },
            gilrs::EventType::ButtonChanged(gilrs::Button::DPadRight, amount, _) => {
                self.dpad.0 = 1.0 * amount;
            },
            gilrs::EventType::ButtonChanged(gilrs::Button::DPadUp, amount, _) => {
                self.dpad.1 = -1.0 * amount;
            },
            gilrs::EventType::ButtonChanged(gilrs::Button::DPadDown, amount, _) => {
                self.dpad.1 = 1.0 * amount;
            },
            _ => {}
        }
    }

    pub fn window_update(&mut self, window_event: &WindowEvent, consumed: bool) {
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
