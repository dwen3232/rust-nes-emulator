use std::mem::transmute;

use log::debug;

use crate::ppu::PPU;

use super::palette;

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
        let index = WIDTH*y + x;
        if index < WIDTH * HEIGHT {
            self.data[WIDTH*y + x] = color;
        }
    }
    
    pub fn render(&mut self, ppu: &PPU) {
        // Renders the background
        let bank = ppu.ppuctrl.get_background_pattern_addr() as usize;
        for i in 0..0x03C0 {
            let tile_n = ppu.vram[i] as usize;
            let tile_range = (bank + 16 * tile_n)..(bank + 16 * (tile_n + 1));
            let tile = &ppu.chr_rom[tile_range];

            let (tile_x, tile_y) = (i % 32, i / 32);

            let palette = Frame::background_palette(ppu, tile_x, tile_y);

            // Render tile
            let (upper, lower) = tile.split_at(8);
            for y in 0..8 {
                let mut hi = upper[y];
                let mut lo = lower[y];
                for x in (0..8).rev() {
                    let hi_bit = (hi & 1) == 1;
                    let lo_bit = (lo & 1) == 1;
                    hi = hi >> 1;
                    lo = lo >> 1;
        
                    let rgb = match (lo_bit, hi_bit) {
                        (false, false) => palette::SYSTEM_PALLETE[palette[0]],
                        (false, true) => palette::SYSTEM_PALLETE[palette[1]],
                        (true, false) => palette::SYSTEM_PALLETE[palette[2]],
                        (true, true) => palette::SYSTEM_PALLETE[palette[3]],
                    };
                    self.set_pixel(8 * tile_x + x, 8 * tile_y + y, rgb);
                }
            }
        }

        // Render sprites
        for i in (0..ppu.oam_data.len()).step_by(4).rev() {
            let tile_y = ppu.oam_data[i] as usize;
            let tile_n = ppu.oam_data[i + 1] as u16;
            let tile_attributes = ppu.oam_data[i + 2];
            let tile_x = ppu.oam_data[i + 3] as usize;

            // 76543210
            // ||||||||
            // ||||||++- Palette (4 to 7) of sprite
            // |||+++--- Unimplemented (read 0)
            // ||+------ Priority (0: in front of background; 1: behind background)
            // |+------- Flip sprite horizontally
            // +-------- Flip sprite vertically
            let flip_vertical = tile_attributes & 0b1000_0000 != 0;
            let flip_horizontal = tile_attributes & 0b0100_0000 != 0;
            let priority = tile_attributes & 0b0010_0000 != 0;
            let palette_idx = tile_attributes & 0b11;

            let palette = Frame::sprite_palette(ppu, palette_idx);
            let bank = ppu.ppuctrl.get_sprite_pattern_addr();

            // TODO: if it's behind background, then isn't it just never shown?
            if !priority {
                let tile_range = (bank + 16 * tile_n) as usize..(bank + 16 * (tile_n + 1)) as usize;
                let tile = &ppu.chr_rom[tile_range];
                let (upper, lower) = tile.split_at(8);
                for y in 0..=7 {
                    let mut hi = upper[y];
                    let mut lo = lower[y];
                    'inner: for x in (0..=7).rev() {
                        let hi_bit = (hi & 1) == 1;
                        let lo_bit = (lo & 1) == 1;
                        hi = hi >> 1;
                        lo = lo >> 1;
                        let rgb = match (lo_bit, hi_bit) {
                            (false, false) => continue 'inner,
                            (false, true) => palette::SYSTEM_PALLETE[palette[1] as usize],
                            (true, false) => palette::SYSTEM_PALLETE[palette[2] as usize],
                            (true, true) => palette::SYSTEM_PALLETE[palette[3] as usize],
                            _ => panic!("impossible"),
                        };
                        match (flip_horizontal, flip_vertical) {
                            (false, false) => self.set_pixel(tile_x + x, tile_y + y, rgb),
                            (false, true) => self.set_pixel(tile_x + x, tile_y + 7 - y, rgb),
                            (true, false) => self.set_pixel(tile_x + 7 - x, tile_y + y, rgb),
                            (true, true) => self.set_pixel(tile_x + 7 - x, tile_y + 7 - y, rgb),
                        }
                    }
                }
            }
            
        }
    }

    pub fn as_bytes_ref(&self) -> &[u8; 3 * WIDTH * HEIGHT] {
        unsafe { transmute(&self.data) }
    }

    fn background_palette(ppu: &PPU, tile_x: usize, tile_y: usize) -> [usize; 4] {
        // Gets the palette for a background tile
        let attribute_offset = 8 * (tile_y / 4) + (tile_x / 4);
        let palette_byte = ppu.vram[0x03C0 + attribute_offset];
        let background_palette = match ((tile_x % 4) / 2, (tile_y % 4) / 2) {
            (0, 0) => palette_byte & 0b11,
            (1, 0) => (palette_byte >> 2) & 0b11,
            (0, 1) => (palette_byte >> 4) & 0b11,
            (1, 1) => (palette_byte >> 6) & 0b11,
            _ => panic!("impossible")
        };
        // $3F01-$3F03	Background palette 0
        // $3F05-$3F07	Background palette 1
        // $3F09-$3F0B	Background palette 2
        // $3F0D-$3F0F	Background palette 3
        let palette_offset = 4 * (background_palette as usize);
        [
            ppu.palette_table[0] as usize,
            ppu.palette_table[palette_offset + 1] as usize,
            ppu.palette_table[palette_offset + 2] as usize,
            ppu.palette_table[palette_offset + 3] as usize,
        ]
    }

    fn sprite_palette(ppu: &PPU, pallete_idx: u8) -> [usize; 4] {
        // Gets the palette for a sprite
        let start = 0x11 + (pallete_idx * 4) as usize;
        [
            0,  // Always transparent
            ppu.palette_table[start] as usize,
            ppu.palette_table[start + 1] as usize,
            ppu.palette_table[start + 2] as usize,
        ]
    }
}