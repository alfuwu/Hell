use std::collections::HashMap;
use winit::keyboard::KeyCode;

#[derive(Clone, Default)]
pub struct Mouse {
    pub x: f64,
    pub y: f64,
    pub left: bool,
    pub right: bool,
    pub middle: bool,
    pub back: bool,
    pub forward: bool
}
impl Mouse {
    pub fn primary_button(&self) -> bool { self.left || self.right || self.middle }
}

#[derive(Clone, Default)]
pub struct Keyboard {
    pub keys: HashMap<KeyCode, bool>
}
impl Keyboard {
    pub fn is_pressed(&self, key: KeyCode) -> bool {
        self.keys.contains_key(&key) && self.keys.get(&key).copied().unwrap_or(false)
    }

    pub(crate) fn set_pressed(&mut self, key: KeyCode, state: bool) {
        self.keys.insert(key, state);
    }
}