
use std::collections::HashSet;

use winit::{
    event::{ElementState, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

pub struct InputManager {
    pressed_keys: HashSet<KeyCode>,
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
        }
    }

    pub fn process_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(key) = event.physical_key {
                    match event.state {
                        ElementState::Pressed => {
                            self.pressed_keys.insert(key);
                        }
                        ElementState::Released => {
                            self.pressed_keys.remove(&key);
                        }
                    }
                }
            }

            WindowEvent::Focused(false) => {
                self.pressed_keys.clear();
            }

            _ => {}
        }
    }

    pub fn is_pressed(&self, key: KeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }
}
