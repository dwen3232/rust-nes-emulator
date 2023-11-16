use crate::rom::{ROM, Mirroring};

use super::PpuState;

pub struct PpuBus<'a, 'b> {
    ppu_state: &'a mut PpuState,
    rom: &'b ROM,
}

impl<'a, 'b> PpuBus<'a, 'b> {
    pub fn new(ppu_state: &'a mut PpuState, rom: &'b ROM) -> Self{
        PpuBus { ppu_state, rom }
    }

    pub fn read_byte(&mut self, index: u16) -> u8 {
        match index {
            0x0000..=0x1FFF => {
                self.rom.chr_rom[index as usize]
            },
            0x2000..=0x2FFF => {
                let vram_index = self.mirror_vram_addr(index);
                self.ppu_state.ram[vram_index as usize]
            },
            0x3000..=0x3EFF => {
                // map to 0x2000...0x2EFF
                let masked_index = index & 0b1110_1111_1111_1111;   
                let vram_index = self.mirror_vram_addr(masked_index);
                self.ppu_state.ram[vram_index as usize]
            },
            0x3F00..=0x3F1F => todo!(),
            0x3F20..=0x3FFF => todo!(),
            _ => panic!("Unexpected address")
        }
    }

    pub fn write_byte(&mut self, index: u16, value: u8) {
        match index {
            0x0000..=0x1FFF => println!("CHR_ROM is read only"),
            // 0x0000..=0x1FFF => panic!("CHR_ROM is read only"),
            0x2000..=0x2FFF => {
                let vram_index = self.mirror_vram_addr(index);
                self.ppu_state.ram[vram_index as usize] = value;
            },
            0x3000..=0x3EFF => {
                // map to 0x2000...0x2EFF
                let masked_index = index & 0b1110_1111_1111_1111;
                let vram_index = self.mirror_vram_addr(masked_index);
                self.ppu_state.ram[vram_index as usize] = value;
            },
            0x3F00..=0x3FFF => {
                // 0x3F20..=0x3FFF mirrors 0x3F00..=0x3FFF
                let masked_index = index & 0b0000_0000_0001_1111;
                let palette_index = match masked_index {
                    0x0010 | 0x0014 | 0x0018 | 0x001C => masked_index - 0x10,
                    _ => masked_index
                };
                self.ppu_state.palette_table[palette_index as usize] = value;
            },
            _ => panic!("Unexpected address")
        }
    }

    fn mirror_vram_addr(&self, addr: u16) -> u16 {
        let vram_index = addr - 0x2000;
        let nametable_index = vram_index / 0x400;

        let mirror_nametable_index = match (&self.rom.mirroring, nametable_index) {
            (Mirroring::Horizontal, 0) => 0,
            (Mirroring::Horizontal, 1) => 0,
            (Mirroring::Horizontal, 2) => 1,
            (Mirroring::Horizontal, 3) => 1,
            (Mirroring::Vertical, 0) => 0,
            (Mirroring::Vertical, 1) => 1,
            (Mirroring::Vertical, 2) => 0,
            (Mirroring::Vertical, 3) => 1,
            _ => panic!("Unexpected mirroring, nametable_index pair")
        };

        (vram_index & 0b1111_0011_1111_1111) | (mirror_nametable_index << 10)
    }
}