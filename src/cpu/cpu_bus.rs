// $0000–$07FF	$0800	2 KB internal RAM
// $0800–$0FFF	$0800	Mirrors of $0000–$07FF
// $1000–$17FF	$0800
// $1800–$1FFF	$0800
// $2000–$2007	$0008	NES PPU registers
// $2008–$3FFF	$1FF8	Mirrors of $2000–$2007 (repeats every 8 bytes)
// $4000–$4017	$0018	NES APU and I/O registers
// $4018–$401F	$0008	APU and I/O functionality that is normally disabled. See CPU Test Mode.
// $4020–$FFFF	$BFE0	Cartridge space: PRG ROM, PRG RAM, and mapper registers (see note)

// More on the cartridge:
// $6000–$7FFF = Battery Backed Save or Work RAM
// $8000–$FFFF = Usual ROM, commonly with Mapper Registers (see MMC1 and UxROM for example)
// UxROM Ref: https://www.nesdev.org/wiki/UxROM


use core::panic;

use crate::rom::ROM;
use crate::controller::Controller;
use crate::ppu::{PPU, PpuState, self};
use crate::common::Memory;

use super::CpuState;

const RAM_START: u16 =      0x0000;
const RAM_END: u16 =        0x1FFF;
const PPU_REG_START: u16 =  0x2000;
const PPU_REG_END: u16 =    0x3FFF;
const APUIO_START: u16 =    0x4000;
const APUIO_END: u16 =      0x401F;
const CART_START: u16 =     0x4020;
const CART_END: u16 =       0xFFFF;

const PRG_ROM_START: u16 =  0x8000;
const PRG_ROM_END: u16 =    0xFFFF;


const RAM_MASK: u16 = (0b1 << 11) -1;
const PPU_MASK: u16 = (0b1 << 3) - 1;

pub struct CpuBus<'a, 'b, 'c, 'd> {
    ram_state: [u8; 0x800],     // 2KB RAM
    pub cpu_state: &'a mut CpuState,    // 
    ppu_state: &'b mut PpuState,
    rom_state: &'c ROM,
    con_state: &'d mut Controller,
    apuio_reg: [u8; 0x20],
}


impl<'a, 'b, 'c, 'd>  CpuBus<'a, 'b, 'c, 'd> {
    pub fn new(cpu_state: &'a mut CpuState, ppu_state: &'b mut PpuState, rom_state: &'c ROM, con_state: &'d mut Controller) -> Self {
        CpuBus { 
            ram_state: [0; 0x800],
            cpu_state,
            ppu_state,
            rom_state,
            con_state: con_state,
            apuio_reg: [0; 0x20],
        }
    }

    pub fn push_to_stack(&mut self, value: u8) {
        // Stack located from 0x100 to 0x1FF, growing downward
        // For push, need to write first, then decrement
        let stack_addr = 0x100 + (self.cpu_state.stack_pointer as u16);
        self.cpu_state.stack_pointer = self.cpu_state.stack_pointer.wrapping_sub(1);
        self.write_byte(stack_addr, value)
    }

    pub fn pop_from_stack(&mut self) -> u8 {
        // For pop, need to increment first, then read
        self.cpu_state.stack_pointer = self.cpu_state.stack_pointer.wrapping_add(1);
        let stack_addr = 0x100 + (self.cpu_state.stack_pointer as u16);
        self.read_byte(stack_addr)
    }
}

impl Memory for CpuBus<'_, '_, '_, '_> {
    fn write_byte(&mut self, index: u16, value: u8) {
        match index {
            RAM_START ..= RAM_END => {
                self.ram_state[(index & RAM_MASK) as usize] = value
            }
            PPU_REG_START ..= PPU_REG_END => {
                let masked_index = index & PPU_MASK;
                match masked_index {
                    0 => self.ppu_state.write_ppuctrl(value),
                    1 => self.ppu_state.write_ppumask(value),
                    2 => panic!("PPUSTATUS is read-only"),
                    3 => self.ppu_state.write_oamaddr(value),
                    4 => self.ppu_state.write_oamdata(value),
                    5 => self.ppu_state.write_ppuscroll(value),
                    6 => self.ppu_state.write_ppuaddr(value),
                    7 => self.ppu_state.write_ppudata(self.rom_state, value),
                    _ => panic!("Invalid PPU_REG index")
                }
            },
            0x4014 => {
                let mut buffer: [u8; 256] = [0; 256];
                let hi: u16 = (value as u16) << 8;
                for i in 0..256u16 {
                    buffer[i as usize] = self.read_byte(hi + i);
                }

                self.ppu_state.write_oamdma(&buffer);
            }
            0x4016 => {
                self.con_state.write(value);
            }
            APUIO_START ..= APUIO_END => {
                let mut index = index - APUIO_START;
                self.apuio_reg[index as usize] = value;
            },
            CART_START ..= CART_END => {
                panic!("Attempted write to read only memory, address {:x}", index);
            }
        }
    }

    fn read_byte(&mut self, index: u16) -> u8 {
        match index {
            RAM_START ..= RAM_END => {
                self.ram_state[(index & RAM_MASK) as usize]
            },
            PPU_REG_START ..= PPU_REG_END => {
                let masked_index = index & PPU_MASK;
                match masked_index {
                    0 => panic!("PPUCTRL is write-only"),
                    1 => panic!("PPUMASK is write-only"),
                    2 => self.ppu_state.read_ppustatus(),
                    3 => panic!("OAMADDR is write-only"),
                    4 => self.ppu_state.read_oamdata(),
                    5 => panic!("PPUSCROLL is write-only"),
                    6 => panic!("PPUADDR is write-only"),
                    7 => self.ppu_state.read_ppudata(self.rom_state),
                    _ => panic!("Invalid PPU_REG index")
                }
            },
            0x4016 => {
                self.con_state.read()
            },
            APUIO_START ..= APUIO_END => {
                let mut index = index - APUIO_START;
                self.apuio_reg[index as usize]
            },
            PRG_ROM_START ..= PRG_ROM_END => {
                let mut index = index - PRG_ROM_START;
                if self.rom_state.prg_rom.len() == 0x4000 && index >= 0x4000 {
                    //mirror if needed
                    index = index % 0x4000;
                }
                self.rom_state.prg_rom[index as usize]
            },
            _ => panic!("Cannot read from {:x}", index)
        }
    }
}


#[cfg(test)]
mod tests {
    // use std::panic::catch_unwind;

    use super::*;
    #[test]
    fn test_all_ram_index_valid() {
        let mut cpu_state = CpuState::new();
        let mut ppu_state = PpuState::new();
        let rom_state = ROM::new();
        let mut con_state = Controller::new();

        let mut mem = CpuBus::new(&mut cpu_state, &mut ppu_state, &rom_state, &mut con_state);

        // populate with some data
        for i in 0..0x800u16 {
            mem.write_byte(i, (i / 8) as u8);
        }

        for i in 0..0x800u16 {
            assert_eq!((i / 8) as u8, mem.read_byte(i));
            assert_eq!((i / 8) as u8, mem.read_byte(i + 0x800));
            assert_eq!((i / 8) as u8, mem.read_byte(i + 0x1000));
            assert_eq!((i / 8) as u8, mem.read_byte(i + 0x1800));
        }
    }

}