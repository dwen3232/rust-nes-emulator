use crate::ppu::PPU;

pub const WIDTH: usize = 256;
pub const HEIGHT: usize = 240;

pub struct Frame {
    pub data: [(u8, u8, u8); WIDTH * HEIGHT]
}

impl Frame {
    pub fn new() -> Self {
        Frame { data: [(0, 0, 0); WIDTH * HEIGHT] }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: (u8, u8, u8)) {
        self.data[WIDTH*y + x] = color;
    }
    
    pub fn render(&mut self, ppu: &PPU) {
        
    }
}