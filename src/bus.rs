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

use crate::cartridge::ROM;

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

#[derive(Debug)]
pub struct Bus {
    ram: [u8; 0x800],   // 2KB RAM
    ppu_reg: [u8; 8],   // 8 PPU registers
    apuio_reg: [u8; 0x20],
    cartridge: ROM,
}


impl Bus {
    pub fn new_empty() -> Self {
        Bus::new(ROM::new_empty())
    }

    pub fn new(cartridge: ROM) -> Self {
        Bus {
            ram: [0; 0x800],
            ppu_reg: [0; 8],
            cartridge: cartridge,
            apuio_reg: [0; 0x20],
        }
    }

    pub fn load_nes(&mut self, path: &str) {
        self.cartridge = ROM::create_from_nes(path).expect("Path does not exist");
    }

    pub fn write_byte(&mut self, index: u16, value: u8) {
        match index {
            RAM_START ..= RAM_END => {
                self.ram[(index & RAM_MASK) as usize] = value
            }
            PPU_REG_START ..= PPU_REG_END => {
                self.ram[(index & PPU_MASK) as usize] = value
            },
            APUIO_START ..= APUIO_END => {
                let mut index = index - APUIO_START;
                self.apuio_reg[index as usize] = value;
            },
            CART_START ..= CART_END => {
                panic!("Attempted write to read only memory, address {:x}", index);
            }
        }
    }

    pub fn read_byte(&self, index: u16) -> u8{
        match index {
            RAM_START ..= RAM_END => {
                self.ram[(index & RAM_MASK) as usize]
            },
            PPU_REG_START ..= PPU_REG_END => {
                self.ram[(index & PPU_MASK) as usize]
            },
            APUIO_START ..= APUIO_END => {
                let mut index = index - APUIO_START;
                self.apuio_reg[index as usize]
            },
            PRG_ROM_START ..= PRG_ROM_END => {
                let mut index = index - PRG_ROM_START;
                if self.cartridge.prg_rom.len() == 0x4000 && index >= 0x4000 {
                    //mirror if needed
                    index = index % 0x4000;
                }
                self.cartridge.prg_rom[index as usize]
            },
            _ => panic!("Cannot read from {:x}", index)
        }
    }

    pub fn read_two_bytes(&self, index: u16) -> u16 {
        let lsb = self.read_byte(index) as u16;
        let msb = self.read_byte(index + 1) as u16;
        (msb << 8) + lsb
    }

}


#[cfg(test)]
mod tests {
    // use std::panic::catch_unwind;

    use super::*;

    #[test]
    fn test_all_ram_index_valid() {
        let mut mem = Bus::new_empty();
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