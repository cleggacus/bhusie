use std::collections::HashSet;

use sdl2::{controller::{Axis, Button}, event::Event};
use winit::{event::{ElementState, KeyEvent, MouseButton, WindowEvent}, keyboard::{KeyCode, PhysicalKey}};

pub struct InputManager {
    keys_down: HashSet<KeyCode>,
    buttons_down: HashSet<Button>,
    left_mouse_down: bool,
    mouse_position: (f64, f64),
    mouse_move: (f64, f64),
    joy_left: (f64, f64),
    joy_right: (f64, f64),
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            keys_down: HashSet::new(),
            buttons_down: HashSet::new(),
            left_mouse_down: false,
            mouse_position: (0.0, 0.0),
            mouse_move: (0.0, 0.0),
            joy_left: (0.0, 0.0),
            joy_right: (0.0, 0.0),
        }
    }

    pub fn is_left_mouse_down(&self) -> bool {
        self.left_mouse_down
    }

    pub fn joy_right(&self) -> (f64, f64) {
        self.joy_right
    }

    pub fn joy_left(&self) -> (f64, f64) {
        self.joy_left
    }

    pub fn mouse_move(&self) -> (f64, f64) {
        self.mouse_move
    }

    pub fn is_button_down(&self, button: Button) -> bool {
        self.buttons_down.contains(&button)
    }

    pub fn is_key_down(&self, key: KeyCode) -> bool {
        self.keys_down.contains(&key)
    }

    pub fn pre_update(&mut self) {
        self.mouse_move = (0.0, 0.0);
    }

    pub fn sdl_update(&mut self, event: &Event) {
        match event {
            Event::ControllerAxisMotion {
                axis: Axis::LeftX,
                value: val, 
                ..
            } => {
                let val = *val as f64 / i16::MAX as f64;
                self.joy_left.0 = val;
            },
            Event::ControllerAxisMotion {
                axis: Axis::LeftY,
                value: val, 
                ..
            } => {
                let val = *val as f64 / i16::MAX as f64;
                self.joy_left.1 = val;
            },
            Event::ControllerAxisMotion {
                axis: Axis::RightX,
                value: val, 
                ..
            } => {
                let val = *val as f64 / i16::MAX as f64;
                self.joy_right.0 = val;
            },
            Event::ControllerAxisMotion {
                axis: Axis::RightY,
                value: val, 
                ..
            } => {
                let val = *val as f64 / i16::MAX as f64;
                self.joy_right.1 = val;
            },
            Event::ControllerButtonDown { button, .. } => {
                self.buttons_down.insert(*button);
            },
            Event::ControllerButtonUp { button, .. } => {
                self.buttons_down.remove(button);
            },
            _ => (),
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
