use crate::{common::Memory, rom::{Mirroring, ROM}};

use super::PpuState;

pub struct PpuBus<'a, 'b> {
    vram: [u8; 0x800],
    palette_table: [u8; 32],
    ppu_state: &'a PpuState,
    rom_state: &'b ROM,
}

impl<'a, 'b> PpuBus<'a, 'b> {
    // PpuState must live at least as long as ROM
    pub fn new(ppu_state: &'a PpuState, rom_state: &'a ROM) -> Self where 'a: 'b {
        PpuBus {
            vram: [0; 0x800],
            palette_table: [0; 32],
            ppu_state,
            rom_state, 
        }
    }

    fn mirror_vram_addr(&self, addr: u16) -> u16 {
        let vram_index = addr - 0x2000;
        let nametable_index = vram_index / 0x400;

        let mirror_nametable_index = match (&self.rom_state.mirroring, nametable_index) {
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

        let new_vram_index = (vram_index & 0b1111_0011_1111_1111) | (mirror_nametable_index << 10);
        return new_vram_index
    }
}
// $0000-$0FFF	$1000	Pattern table 0
// $1000-$1FFF	$1000	Pattern table 1
// $2000-$23FF	$0400	Nametable 0
// $2400-$27FF	$0400	Nametable 1
// $2800-$2BFF	$0400	Nametable 2
// $2C00-$2FFF	$0400	Nametable 3
// $3000-$3EFF	$0F00	Mirrors of $2000-$2EFF
// $3F00-$3F1F	$0020	Palette RAM indexes
// $3F20-$3FFF	$00E0	Mirrors of $3F00-$3F1F
// TODO: maybe move this into a separate Bus struct?
impl Memory for PpuBus<'_, '_> {
    fn read_byte(&mut self, index: u16) -> u8 {
        match index {
            0x0000..=0x1FFF => {
                self.rom_state.chr_rom[index as usize]
            },
            0x2000..=0x2FFF => {
                let vram_index = self.mirror_vram_addr(index);
                self.vram[vram_index as usize]
            },
            0x3000..=0x3EFF => {
                // map to 0x2000...0x2EFF
                let masked_index = index & 0b1110_1111_1111_1111;   
                let vram_index = self.mirror_vram_addr(masked_index);
                self.vram[vram_index as usize]
            },
            0x3F00..=0x3F1F => todo!(),
            0x3F20..=0x3FFF => todo!(),
            _ => panic!("Unexpected address")
        }
    }

    fn write_byte(&mut self, index: u16, value: u8) {
        match index {
            0x0000..=0x1FFF => panic!("CHR_ROM is read only"),
            0x2000..=0x2FFF => {
                let vram_index = self.mirror_vram_addr(index);
                self.vram[vram_index as usize] = value;
            },
            0x3000..=0x3EFF => {
                // map to 0x2000...0x2EFF
                let masked_index = index & 0b1110_1111_1111_1111;
                let vram_index = self.mirror_vram_addr(masked_index);
                self.vram[vram_index as usize] = value;
            },
            0x3F00..=0x3FFF => {
                // 0x3F20..=0x3FFF mirrors 0x3F00..=0x3FFF
                let masked_index = index & 0b0000_0000_0001_1111;
                let palette_index = match masked_index {
                    0x0010 | 0x0014 | 0x0018 | 0x001C => masked_index - 0x10,
                    _ => masked_index
                };
                self.palette_table[palette_index as usize] = value;
            },
            _ => panic!("Unexpected address")
        }
    }
}