use crate::audio::Audio;
use crate::cpu::CPU;
use crate::display::Display;
use crate::input::Input;

pub struct Chip8 {
    cpu: CPU,
    display: Display,
    audio: Audio,
    input: Input,
}

impl Chip8 {
    pub fn new(program: &[u8]) -> Self {
        Self {
            cpu: CPU::new(program),
            display: Display::new(),
            audio: Audio::new(),
            input: Input::new(),
        }
    }

    pub fn cycle(&mut self) {
        self.cpu.cycle(&mut self.display, &self.input);
    }

    pub fn tick(&mut self) {
        self.cpu.tick();
        if self.cpu.should_play_sound() {
            self.audio.play();
        } else {
            self.audio.pause();
        }
    }

    pub fn display(&self) -> &Display {
        &self.display
    }

    pub fn key_pressed(&mut self, key: u8) {
        self.input.key_pressed(key)
    }

    pub fn key_released(&mut self, key: u8) {
        self.input.key_released(key)
    }
}
