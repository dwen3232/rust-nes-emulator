use std::mem::transmute;

// use crate::ppu::PPU;

use crate::{
    ppu::PpuState,
    rom::{Mirroring, ROM},
};

use super::palette;

pub const WIDTH: usize = 256;
pub const HEIGHT: usize = 240;

#[derive(Debug, Clone, Copy)]
pub struct Frame {
    pub data: [(u8, u8, u8); WIDTH * HEIGHT],
}

struct View {
    x1: usize,
    y1: usize,
    x2: usize,
    y2: usize,
}

impl View {
    pub fn new(x1: usize, y1: usize, x2: usize, y2: usize) -> Self {
        View { x1, y1, x2, y2 }
    }
}

impl Default for Frame {
    fn default() -> Self {
        Self::new()
    }
}

impl Frame {
    pub fn new() -> Self {
        Frame {
            data: [(0, 0, 0); WIDTH * HEIGHT],
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: (u8, u8, u8)) {
        let index = WIDTH * y + x;
        if index < WIDTH * HEIGHT {
            self.data[WIDTH * y + x] = color;
        }
    }

    pub fn as_bytes_ref(&self) -> &[u8; 3 * WIDTH * HEIGHT] {
        unsafe { transmute(&self.data) }
    }

    // TODO: first few rendered lines are usually invisible, maybe implement that?
    pub fn render(&mut self, ppu: &PpuState, rom: &ROM) {
        self.render_background(ppu, rom);
        self.render_sprites(ppu, rom);
    }

    /// Helper function for rendering all background tiles
    fn render_background(&mut self, ppu: &PpuState, rom: &ROM) {
        let (scroll_x, scroll_y) = ppu.ppuscroll.read();
        // println!("Scroll: {} {}", scroll_x, scroll_y);
        let (first_name_table, second_name_table) =
            match (&rom.mirroring, ppu.ppuctrl.get_name_table_addr()) {
                (Mirroring::Vertical, 0x2000)
                | (Mirroring::Vertical, 0x2800)
                | (Mirroring::Horizontal, 0x2000)
                | (Mirroring::Horizontal, 0x2400) => (&ppu.ram[0..0x400], &ppu.ram[0x400..0x800]),
                (Mirroring::Vertical, 0x2400)
                | (Mirroring::Vertical, 0x2C00)
                | (Mirroring::Horizontal, 0x2800)
                | (Mirroring::Horizontal, 0x2C00) => (&ppu.ram[0x400..0x800], &ppu.ram[0..0x400]),
                (_, _) => {
                    panic!("Not supported mirroring type {:?}", rom.mirroring);
                }
            };

        // Renders ther first name table
        let first_name_table_view = View::new(scroll_x, scroll_y, 256, 240);
        self.render_name_table(
            ppu,
            rom,
            first_name_table,
            first_name_table_view,
            -(scroll_x as isize),
            -(scroll_y as isize),
        );

        // Render second name table
        // TODO: what should happen if both scroll_x and scroll_y are > 0?
        // TODO: refactor this, this is kind of ugly
        // if scroll_x > 0 {
        let second_name_table_view = View::new(0, 0, scroll_x, 240);
        self.render_name_table(
            ppu,
            rom,
            second_name_table,
            second_name_table_view,
            (256 - scroll_x) as isize,
            0,
        );
        // } else if scroll_y > 0 {
        //     let second_name_table_view = View::new(0, 0, 256, scroll_y);
        //     self.render_name_table(
        //         ppu,
        //         rom,
        //         second_name_table,
        //         second_name_table_view,
        //         0,
        //         (240 - scroll_y) as isize,
        //     );
        // }
    }

    /// Helper function for rendering a name table to the screen (taking scrolling into account)
    fn render_name_table(
        &mut self,
        ppu: &PpuState,
        rom: &ROM,
        name_table: &[u8],
        view: View,
        shift_x: isize,
        shift_y: isize,
    ) {
        let attribute_table = &name_table[0x3c0..0x400];
        let bank = ppu.ppuctrl.get_background_pattern_addr() as usize;
        for (i, &tile_n) in name_table.iter().enumerate().take(0x03C0) {
            let tile_n = tile_n as usize;
            let tile_range = (bank + 16 * tile_n)..(bank + 16 * (tile_n + 1));
            let tile = &rom.chr_rom[tile_range];

            let (tile_x, tile_y) = (i % 32, i / 32);

            let palette = Self::background_palette(ppu, attribute_table, tile_x, tile_y);

            // Render tile
            let (upper, lower) = tile.split_at(8);
            for y in 0..8 {
                let mut hi = upper[y];
                let mut lo = lower[y];
                for x in (0..8).rev() {
                    let hi_bit = (hi & 1) == 1;
                    let lo_bit = (lo & 1) == 1;
                    hi >>= 1;
                    lo >>= 1;

                    let rgb = match (lo_bit, hi_bit) {
                        (false, false) => palette::SYSTEM_PALLETE[palette[0]],
                        (false, true) => palette::SYSTEM_PALLETE[palette[1]],
                        (true, false) => palette::SYSTEM_PALLETE[palette[2]],
                        (true, true) => palette::SYSTEM_PALLETE[palette[3]],
                    };
                    let pixel_x = 8 * tile_x + x;
                    let pixel_y = 8 * tile_y + y;
                    if pixel_x >= view.x1
                        && pixel_x < view.x2
                        && pixel_y >= view.y1
                        && pixel_y < view.y2
                    {
                        self.set_pixel(
                            (shift_x + pixel_x as isize) as usize,
                            (shift_y + pixel_y as isize) as usize,
                            rgb,
                        );
                    }
                    // TEMPORARY: just drawing me some lines
                    if (pixel_x == view.x1 || pixel_x == view.x2) && pixel_y >= view.y1
                    && pixel_y < view.y2
                    {
                        self.set_pixel(
                            (shift_x + pixel_x as isize) as usize,
                            (shift_y + pixel_y as isize) as usize,
                            (255, 0, 0),
                        );
                    }
                }
            }
        }
    }

    /// Helper method for rendering all sprite tiles
    fn render_sprites(&mut self, ppu: &PpuState, rom: &ROM) {
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
                let tile = &rom.chr_rom[tile_range];
                let (upper, lower) = tile.split_at(8);
                for y in 0..=7 {
                    let mut hi = upper[y];
                    let mut lo = lower[y];
                    'inner: for x in (0..=7).rev() {
                        let hi_bit = (hi & 1) == 1;
                        let lo_bit = (lo & 1) == 1;
                        hi >>= 1;
                        lo >>= 1;
                        let rgb = match (lo_bit, hi_bit) {
                            (false, false) => continue 'inner,
                            (false, true) => palette::SYSTEM_PALLETE[palette[1]],
                            (true, false) => palette::SYSTEM_PALLETE[palette[2]],
                            (true, true) => palette::SYSTEM_PALLETE[palette[3]],
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

    /// Helper function for retrieving the palette for a background tile
    fn background_palette(
        ppu: &PpuState,
        attribute_table: &[u8],
        tile_x: usize,
        tile_y: usize,
    ) -> [usize; 4] {
        let attribute_offset = 8 * (tile_y / 4) + (tile_x / 4);
        let palette_byte = attribute_table[attribute_offset];
        let background_palette = match ((tile_x % 4) / 2, (tile_y % 4) / 2) {
            (0, 0) => palette_byte & 0b11,
            (1, 0) => (palette_byte >> 2) & 0b11,
            (0, 1) => (palette_byte >> 4) & 0b11,
            (1, 1) => (palette_byte >> 6) & 0b11,
            _ => panic!("impossible"),
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

    // Helper function for retrieving the pallete for a sprite tile
    fn sprite_palette(ppu: &PpuState, pallete_idx: u8) -> [usize; 4] {
        // Gets the palette for a sprite
        let start = 0x11 + (pallete_idx * 4) as usize;
        [
            0, // Always transparent
            ppu.palette_table[start] as usize,
            ppu.palette_table[start + 1] as usize,
            ppu.palette_table[start + 2] as usize,
        ]
    }
}
