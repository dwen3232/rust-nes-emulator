// $0000–$07FF	$0800	2 KB internal RAM
// $0800–$0FFF	$0800	Mirrors of $0000–$07FF
// $1000–$17FF	$0800
// $1800–$1FFF	$0800
// $2000–$2007	$0008	NES PPU registers
// $2008–$3FFF	$1FF8	Mirrors of $2000–$2007 (repeats every 8 bytes)
// $4000–$4017	$0018	NES APU and I/O registers
// $4018–$401F	$0008	APU and I/O functionality that is normally disabled. See CPU Test Mode.
// $4020–$FFFF	$BFE0	Cartridge space: PRG ROM, PRG RAM, and mapper registers (see note)

const RAM_START: u16 = 0x0000;
const RAM_END: u16 = 0x1FFF;
const PPU_REG_START: u16 = 0x2000;
const PPU_REG_END: u16 = 0x3FFF;
const APUIO_START: u16 = 0x4000;
const APUIO_END: u16 = 0x401F;
const CART_START: u16 = 0x4020;
const CART_END: u16 = 0xFFFF;


const RAM_MASK: u16 = (0b1 << 11) -1;
const PPU_MASK: u16 = (0b1 << 3) - 1;

pub struct Bus {
    ram: [u8; 0x800],   // 2KB RAM
    ppu_reg: [u8; 8],   // 8 PPU registers
}


impl Bus {
    pub fn new() -> Self {
        Bus {
            ram: [0; 0x800],
            ppu_reg: [0; 8],
        }
    }

    pub fn write(&mut self, index: u16, value: u8) {
        match index {
            RAM_START ..= RAM_END => {
                self.ram[(index & RAM_MASK) as usize] = value
            }
            PPU_REG_START ..= PPU_REG_END => {
                self.ram[(index & PPU_MASK) as usize] = value
            },
            APUIO_START ..= APUIO_END => {
                todo!("Not implemented")
            },
            CART_START ..= CART_END => {
                todo!("Not implemented")
            }
        }
    }

    pub fn read(&self, index: u16) -> u8{
        match index {
            RAM_START ..= RAM_END => {
                self.ram[(index & RAM_MASK) as usize]
            },
            PPU_REG_START ..= PPU_REG_END => {
                self.ram[(index & PPU_MASK) as usize]
            },
            APUIO_START ..= APUIO_END => {
                todo!("Not implemented")
            },
            CART_START ..= CART_END => {
                todo!("Not implemented")
            }
        }
    }
}


#[cfg(test)]
mod tests {
    // use std::panic::catch_unwind;

    use super::*;

    #[test]
    fn test_all_index_valid() {
        let mut mem = Bus::new();
        // populate with some data
        for i in 0..0x800u16 {
            mem.write(i, (i / 8) as u8);
        }

        for i in 0..0x800u16 {
            assert_eq!((i / 8) as u8, mem.read(i));
            assert_eq!((i / 8) as u8, mem.read(i + 0x800));
            assert_eq!((i / 8) as u8, mem.read(i + 0x1000));
            assert_eq!((i / 8) as u8, mem.read(i + 0x1800));
        }
    }

}