
// ~~~FULL FILE FORMAT:
// Header (16 bytes)
// Trainer, if present (0 or 512 bytes)
// PRG ROM data (16384 * x bytes)
// CHR ROM data, if present (8192 * y bytes)
// PlayChoice INST-ROM, if present (0 or 8192 bytes)
// PlayChoice PROM, if present (16 bytes Data, 16 bytes CounterOut) (this is often missing; see PC10 ROM-Images for details)

// $6000–$7FFF = Battery Backed Save or Work RAM
// $8000–$FFFF = Usual ROM, commonly with Mapper Registers (see MMC1 and UxROM for example)
// UxROM Ref: https://www.nesdev.org/wiki/UxROM

use std::fs::{read};

const HEADER_TAG: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];
const PRG_ROM_PAGE_SIZE: usize = 16384; // 16 KB page size
const CHR_ROM_PAGE_SIZE: usize = 8192;  // 8 KB page size

// For flag 6
const MIRROR_MASK: u8 =      0b0000_0001;
const CARTRIDGE_MASK: u8 =   0b0000_0010;
const TRAINER_MASK: u8 =     0b0000_0100;
const FOUR_SCREEN_MASK: u8 = 0b0000_1000;

// For flag 7
const VS_UNISYS_MASK: u8 =   0b0000_0001;
const PLAYCHOICE_MASK: u8 =  0b0000_0010;

pub const PRG_ROM_SIZE: usize = PRG_ROM_PAGE_SIZE * u8::MAX as usize;
pub const CHR_ROM_SIZE: usize = CHR_ROM_PAGE_SIZE * u8::MAX as usize;


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mirroring {
    Vertical, Horizontal, FourScreen,
}

// Representation for a cartridge. Uses .nes file format
#[derive(Debug, Clone)]
pub struct ROM {
    pub mirroring: Mirroring,
    pub mapper: u8,
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    // pub prg_rom: [u8; PRG_ROM_SIZE],
    // pub chr_rom: [u8; CHR_ROM_SIZE],
}

impl Default for ROM {
    fn default() -> Self {
        Self::new()
    }
}

impl ROM {
    pub fn new() -> Self {
        // Creates ROM with no data, useful for testing other components
        ROM {
            mirroring: Mirroring::Horizontal,
            mapper: 0,
            prg_rom: vec![],
            chr_rom: vec![],
            // prg_rom: [0; PRG_ROM_SIZE],
            // chr_rom: [0; CHR_ROM_SIZE],
        }
    }

    pub fn create_from_nes(path: &str) -> Result<Self, String> {
        // Creates a ROM with data loaded from a .nes file
        let program = read(path).expect("Path does not exist");
        Self::from(program)
    }

    pub fn from(raw: Vec<u8>) -> Result<Self, String>{
        // First, decode the header
        // ~~~HEADER FORMAT:
        // 0-3	Constant $4E $45 $53 $1A (ASCII "NES" followed by MS-DOS end-of-file)
        // 4	Size of PRG ROM in 16 KB units
        // 5	Size of CHR ROM in 8 KB units (value 0 means the board uses CHR RAM)
        // 6	Flags 6 – Mapper, mirroring, battery, trainer
        // 7	Flags 7 – Mapper, VS/Playchoice, NES 2.0
        // 8	Flags 8 – PRG-RAM size (rarely used extension)
        // 9	Flags 9 – TV system (rarely used extension)
        // 10	Flags 10 – TV system, PRG-RAM presence (unofficial, rarely used extension)
        // 11-15	Unused padding (should be filled with zero, but some rippers put their name across bytes 7-15)
        // TODO: only handling flag 6 and 7, since 8, 9, 10 are rarely used, may need to implement in future

        if raw[..4] != HEADER_TAG {
            return Err("Header tag invalid".to_string());
        }
        let prg_rom_size = PRG_ROM_PAGE_SIZE * (raw[4] as usize);
        let chr_rom_size = CHR_ROM_PAGE_SIZE * (raw[5] as usize);
        println!{"Found prg_rom_size of {:x}, or {} pages", prg_rom_size, raw[4]}
        // ~~FLAG 6:
        // 76543210
        // ||||||||
        // |||||||+- Mirroring: 0: horizontal (vertical arrangement) (CIRAM A10 = PPU A11)
        // |||||||              1: vertical (horizontal arrangement) (CIRAM A10 = PPU A10)
        // ||||||+-- 1: Cartridge contains battery-backed PRG RAM ($6000-7FFF) or other persistent memory
        // |||||+--- 1: 512-byte trainer at $7000-$71FF (stored before PRG data)
        // ||||+---- 1: Ignore mirroring control or above mirroring bit; instead provide four-screen VRAM
        // ++++----- Lower nybble of mapper number
        // Right now, only checking for mirror, four screen flags
        let flag_6_byte = raw[6];
        let mirror = flag_6_byte & MIRROR_MASK != 0;
        let cartridge = flag_6_byte & CARTRIDGE_MASK != 0;
        let trainer = flag_6_byte & TRAINER_MASK != 0;
        let four_screen = flag_6_byte & FOUR_SCREEN_MASK != 0;
        let mapper_number_lsb = (flag_6_byte >> 4) & 0b0000_1111;

        // ~~FLAG 7
        // 76543210
        // ||||||||
        // |||||||+- VS Unisystem
        // ||||||+-- PlayChoice-10 (8 KB of Hint Screen data stored after CHR data)
        // ||||++--- If equal to 2, flags 8-15 are in NES 2.0 format
        // ++++----- Upper nybble of mapper number
        let flag_7_byte = raw[7];
        // let vs_unisys = flag_7_byte & VS_UNISYS_MASK != 0;
        // let playchoice = flag_7_byte & PLAYCHOICE_MASK != 0;
        let nes_format = (flag_7_byte >> 2) & 0b0000_0011;
        let mapper_number_msb = flag_7_byte & 0b1111_0000;  // Don't shift this

        if nes_format != 0 {
            return Err("Currently do not support NES2.0 format".to_string());
        }

        let mirroring = match (four_screen, mirror) {
            (true, _) => Mirroring::FourScreen,
            (_, true) => Mirroring::Vertical,
            (_, _)    => Mirroring::Horizontal,
        };
        let mapper = mapper_number_msb + mapper_number_lsb;
        // If there is a trainer, then the trainer block is 512, otherwise 0
        let prg_rom_start = 16 + if trainer{ 512 } else {0};
        // chr_rom starts after prg_rom
        let chr_rom_start = prg_rom_start + prg_rom_size;

        Ok(ROM {
            mirroring,
            mapper,
            prg_rom: raw[prg_rom_start .. (prg_rom_start + prg_rom_size)].to_vec(),
            chr_rom: raw[chr_rom_start .. (chr_rom_start + chr_rom_size)].to_vec(),
        })
    }


}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialization() {
        let rom = ROM::new();
        assert_eq!(0, rom.mapper)
    }
}