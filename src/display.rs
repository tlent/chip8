const WIDTH: usize = 64;
const HEIGHT: usize = 32;

#[derive(Debug, Clone)]
pub struct Display(Vec<bool>);

impl Display {
    pub fn new() -> Self {
        Self(vec![false; WIDTH * HEIGHT])
    }

    pub fn get_pixel(&self, x: u8, y: u8) -> bool {
        let x = x as usize;
        let y = y as usize;
        if x >= WIDTH || y >= HEIGHT {
            panic!("Pixel coordinate out of range: {:?}", (x, y));
        }
        let i = (HEIGHT - 1 - y) * WIDTH + x;
        self.0[i]
    }

    pub fn set_pixel(&mut self, x: u8, y: u8, value: bool) {
        let x = x as usize;
        let y = y as usize;
        if x >= WIDTH || y >= HEIGHT {
            panic!("Pixel coordinate out of range: {:?}", (x, y));
        }
        let i = (HEIGHT - 1 - y) * WIDTH + x;
        let pixel = &mut self.0[i];
        *pixel = value;
    }

    pub fn clear(&mut self) {
        for v in self.0.iter_mut() {
            *v = false;
        }
    }

    pub fn dimensions(&self) -> (u8, u8) {
        (WIDTH as u8, HEIGHT as u8)
    }

    pub fn into_inner(self) -> Vec<bool> {
        self.0
    }
}
