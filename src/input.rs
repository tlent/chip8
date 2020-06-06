pub struct Input([bool; 16]);

impl Input {
    pub fn new() -> Self {
        Self([false; 16])
    }

    pub fn key_pressed(&mut self, key: u8) {
        self.0[key as usize] = true;
    }

    pub fn key_released(&mut self, key: u8) {
        self.0[key as usize] = false;
    }

    pub fn is_key_pressed(&self, key: u8) -> bool {
        self.0[key as usize]
    }

    pub fn get_pressed_key(&self) -> Option<u8> {
        for key in 0..self.0.len() {
            if self.0[key] {
                return Some(key as u8);
            }
        }
        None
    }
}
