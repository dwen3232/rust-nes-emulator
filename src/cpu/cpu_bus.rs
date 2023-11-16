use crate::{rom::ROM, controller::Controller, ppu::{PpuState, PpuAction}};

use super::{CpuState, CpuAction};

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
    cpu_state: &'a mut CpuState,
    ppu_state: &'b mut PpuState,
    controller: &'c mut Controller,
    rom: &'d ROM,
    // TODO: I think this needs to be moved somewhere else?
    apuio_reg: [u8; 0x20],
}

// impl From<CpuAction> for CpuBus {
//     fn from(item: CpuAction) -> Self {
//         let CpuAction { cpu_state, ppu_state, controller, rom } = *self;
//         CpuBus::new(cpu_state, ppu_state, controller, rom)
//     }
// }

impl<'a, 'b, 'c, 'd> CpuBus<'a, 'b, 'c, 'd> {
    pub fn new(
        cpu_state: &'a mut CpuState,
        ppu_state: &'b mut PpuState,
        controller: &'c mut Controller,
        rom: &'d ROM,
    ) -> Self {
        // TODO: apuio_reg should also be a slice probably
        CpuBus { cpu_state, ppu_state, controller, rom, apuio_reg: [0; 0x20] } 
    }

    /// Read a byte from the program counter, incrementing it
    pub fn read_byte_from_pc(&mut self) -> u8 {
        let read_addr = self.cpu_state.program_counter;
        self.cpu_state.program_counter += 1;
        self.read_byte(read_addr)
    }

    /// Reads two bytes from the program counter, incrementing it twice
    pub fn read_two_bytes_from_pc(&mut self) -> u16 {
        let read_addr = self.cpu_state.program_counter;
        self.cpu_state.program_counter += 2;
        self.read_two_bytes(read_addr)
    }

    /// Reads two bytes from a location
    pub fn read_two_bytes(&mut self, index: u16) -> u16 {
        let lsb = self.read_byte(index) as u16;
        let msb = self.read_byte(index + 1) as u16;
        
        (msb << 8) + lsb
    }

    /// Reads two bytes from a location, looping back to the start of the page if on a boundary
    pub fn read_two_page_bytes(&mut self, index: u16) -> u16 {
        let lsb = self.read_byte(index) as u16;
        let msb = self.read_byte((index as u8).wrapping_add(1) as u16) as u16;
        
        (msb << 8) + lsb
    }

    /// Writes a byte to a location
    pub fn write_byte(&mut self, index: u16, value: u8) {
        match index {
            RAM_START ..= RAM_END => {
                self.cpu_state.ram[(index & RAM_MASK) as usize] = value
            }
            PPU_REG_START ..= PPU_REG_END => {
                let masked_index = index & PPU_MASK;
                let mut ppu_action = PpuAction::new(self.ppu_state, self.rom);
                match masked_index {
                    // TODO: update this to use PPUAction
                    0 => ppu_action.write_ppuctrl(value),
                    1 => ppu_action.write_ppumask(value),
                    2 => panic!("PPUSTATUS is read-only"),
                    3 => ppu_action.write_oamaddr(value),
                    4 => ppu_action.write_oamdata(value),
                    5 => ppu_action.write_ppuscroll(value),
                    6 => ppu_action.write_ppuaddr(value),
                    7 => ppu_action.write_ppudata(value),
                    _ => panic!("Invalid PPU_REG index")
                }
            },
            0x4014 => {
                let mut buffer: [u8; 256] = [0; 256];
                let hi: u16 = (value as u16) << 8;
                for i in 0..256u16 {
                    buffer[i as usize] = self.read_byte(hi + i);
                }
                let mut ppu_action = PpuAction::new(self.ppu_state, self.rom);
                ppu_action.write_oamdma(&buffer);
            }
            0x4016 => {
                self.controller.write(value);
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

    /// Reads a byte from a location, may have side effects from triggering PPU behavior
    pub fn read_byte(&mut self, index: u16) -> u8 {
        match index {
            RAM_START ..= RAM_END => {
                self.cpu_state.ram[(index & RAM_MASK) as usize]
            },
            PPU_REG_START ..= PPU_REG_END => {
                let masked_index = index & PPU_MASK;
                let mut ppu_action = PpuAction::new(self.ppu_state, self.rom);
                match masked_index {
                    0 => panic!("PPUCTRL is write-only"),
                    1 => panic!("PPUMASK is write-only"),
                    2 => ppu_action.read_ppustatus(),
                    3 => panic!("OAMADDR is write-only"),
                    4 => ppu_action.read_oamdata(),
                    5 => panic!("PPUSCROLL is write-only"),
                    6 => panic!("PPUADDR is write-only"),
                    7 => ppu_action.read_ppudata(),
                    _ => panic!("Invalid PPU_REG index")
                }
            },
            0x4016 => {
                self.controller.read()
            },
            APUIO_START ..= APUIO_END => {
                let mut index = index - APUIO_START;
                self.apuio_reg[index as usize]
            },
            PRG_ROM_START ..= PRG_ROM_END => {
                let mut index = index - PRG_ROM_START;
                if self.rom.prg_rom.len() == 0x4000 && index >= 0x4000 {
                    //mirror if needed
                    index %= 0x4000;
                }
                self.rom.prg_rom[index as usize]
            },
            _ => panic!("Cannot read from {:x}", index)
        }
    }

    /// Reads a byte from a location with no side effects!
    pub fn peek_byte(&self, index: u16) -> u8 {
        match index {
            RAM_START ..= RAM_END => {
                self.cpu_state.ram[(index & RAM_MASK) as usize]
            },
            PPU_REG_START ..= PPU_REG_END => {
                let masked_index = index & PPU_MASK;
                panic!("Invalid PPU_REG index")
            },
            0x4016 => {
                self.controller.peek()
            },
            APUIO_START ..= APUIO_END => {
                let mut index = index - APUIO_START;
                self.apuio_reg[index as usize]
            },
            PRG_ROM_START ..= PRG_ROM_END => {
                let mut index = index - PRG_ROM_START;
                if self.rom.prg_rom.len() == 0x4000 && index >= 0x4000 {
                    //mirror if needed
                    index %= 0x4000;
                }
                self.rom.prg_rom[index as usize]
            },
            _ => panic!("Cannot read from {:x}", index)
        }
    }

    pub fn peek_two_bytes(&self, index: u16) -> u16 {
        let lsb = self.peek_byte(index) as u16;
        let msb = self.peek_byte(index + 1) as u16;
        
        (msb << 8) + lsb
    }
}