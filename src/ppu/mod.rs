mod registers;
use core::panic;

use registers::{
    PpuControl,
    PpuMask,
    PpuStatus,
    OamAddr,
    OamData,
    PpuScroll,
    PpuAddr
};

use super::traits::Memory;
use super::cartridge::Mirroring;


// PPUCTRL	$2000	VPHB SINN	NMI enable (V), PPU master/slave (P), sprite height (H), background tile select (B), sprite tile select (S), increment mode (I), nametable select (NN)
// PPUMASK	$2001	BGRs bMmG	color emphasis (BGR), sprite enable (s), background enable (b), sprite left column enable (M), background left column enable (m), greyscale (G)
// PPUSTATUS	$2002	VSO- ----	vblank (V), sprite 0 hit (S), sprite overflow (O); read resets write pair for $2005/$2006
// OAMADDR	$2003	aaaa aaaa	OAM read/write address
// OAMDATA	$2004	dddd dddd	OAM data read/write
// PPUSCROLL	$2005	xxxx xxxx	fine scroll position (two writes: X scroll, Y scroll)
// PPUADDR	$2006	aaaa aaaa	PPU read/write address (two writes: most significant byte, least significant byte)
// PPUDATA	$2007	dddd dddd	PPU data read/write
// OAMDMA	$4014	aaaa aaaa	OAM DMA high address

// OAM data, 64 sprites each occupying 4 bytes, so 256 bytes in total
#[derive(Debug)]
pub struct PPU {
    pub chr_rom: Vec<u8>,
    pub vram: [u8; 0x800],
    pub oam_data: [u8; 256],
    // pub palette_table: [u8; 32],
    // registers
    ppuctrl: PpuControl,
    ppumask: PpuMask,
    ppustatus:PpuStatus,
    oamaddr: OamAddr,
    // oamdata: OamData,
    ppuscroll: PpuScroll,
    ppuaddr: PpuAddr,
    ppudata: u8,

    pub mirroring: Mirroring,

    // TODO: Can these be smaller?
    cycle_counter: usize,
    cur_scanline: usize, 

    pub nmi_interrupt_signal: Option<()>,
}

impl PPU {
    pub fn new(chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        PPU {
            chr_rom: chr_rom,
            vram: [0; 0x800],
            oam_data: [0; 256],
            ppuctrl: PpuControl::from_bits_retain(0),
            ppumask: PpuMask::from_bits_retain(0),
            ppustatus: PpuStatus::from_bits_retain(0),
            oamaddr: OamAddr::new(),
            ppuscroll: PpuScroll::new(),
            ppuaddr: PpuAddr::new(),
            ppudata: 0,
            mirroring: mirroring,
            cycle_counter: 0,
            cur_scanline: 0,
            nmi_interrupt_signal: None,
        }
    }

    pub fn new_empty_chr_rom(mirroring: Mirroring) -> Self {
        PPU::new(vec![0; 2048], mirroring)
    }

    pub fn write_ppuctrl(&mut self, data: u8) {
        let prev_is_generate_nmi = self.ppuctrl.is_generate_nmi();
        self.ppuctrl.write(data);
        let is_vblank_started = self.ppustatus.is_vblank_started();
        let cur_is_generate_nmi = self.ppuctrl.is_generate_nmi();
        // Set NMI Interrupt signal if PPU is in VBLANK and GENERATE_NMI changes from 0 to 1
        if !prev_is_generate_nmi && cur_is_generate_nmi && is_vblank_started {
            self.nmi_interrupt_signal = Some(())
        }
    }

    pub fn write_ppumask(&mut self, data: u8) {
        self.ppumask.write(data);
    }

    pub fn read_ppustatus(&mut self) -> u8 {
        let bits = self.ppustatus.bits();
        self.ppustatus.remove(PpuStatus::VBLANK_STARTED);
        self.ppuscroll.reset();
        self.ppuaddr.reset();
        bits
    }

    pub fn write_oamaddr(&mut self, data: u8) {
        self.oamaddr.write(data);
    }

    pub fn write_oamdata(&mut self, data: u8) {
        self.oam_data[self.oamaddr.read() as usize] = data;
        self.oamaddr.increment();
    }

    pub fn write_oamdma(&mut self, data: &[u8; 256]) {
        for byte in data.iter() {
            self.oam_data[self.oamaddr.read() as usize] = *byte;
            self.oamaddr.increment();
        }
    }

    pub fn read_oamdata(&self) -> u8 {
        self.oam_data[self.oamaddr.read() as usize]
    }

    pub fn write_ppuscroll(&mut self, data: u8) {
        self.ppuscroll.write(data);
    }

    pub fn write_ppuaddr(&mut self, data: u8) {
        self.ppuaddr.write(data);
    }

    pub fn read_ppudata(&mut self) -> u8 {
        let addr = self.ppuaddr.read();
        // Retrieve previous value in buffer
        let result = self.ppudata;
        // Store in ppudata as buffer
        self.ppudata = self.read_byte(addr);
        // Increment address
        let inc_value = self.ppuctrl.get_vram_addr_inc_value();
        self.ppuaddr.increment(inc_value);
        return result;
    }

    pub fn write_ppudata(&mut self, data: u8) {
        let addr = self.ppuaddr.read();
        self.write_byte(addr, data);
        // Increment address
        let inc_value = self.ppuctrl.get_vram_addr_inc_value();
        self.ppuaddr.increment(inc_value);
    }

    fn mirror_vram_addr(&self, addr: u16) -> u16 {
        let vram_index = addr - 0x2000;
        let nametable_index = vram_index / 0x400;

        let mirror_nametable_index = match (&self.mirroring, nametable_index) {
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

    pub fn increment_cycle_counter(&mut self, cycles: u8) {
        self.cycle_counter += cycles as usize;
        // cycle_counter loops back to 0 at 341 and increments cur_scalenline
        if self.cycle_counter < 341 {
            return;
        }
        self.cycle_counter = self.cycle_counter - 341;
        self.cur_scanline += 1;

        if self.cur_scanline == 241 {
            self.ppustatus.set_vblank_started(true);
            self.ppustatus.set_sprite_zero_hit(false);
            if self.ppuctrl.is_generate_nmi() {
                self.nmi_interrupt_signal = Some(());
            }
        } else if self.cur_scanline >= 262 {
            self.cur_scanline = 0;
            self.nmi_interrupt_signal = None;
            self.ppustatus.set_vblank_started(false);
            self.ppustatus.set_sprite_zero_hit(false);
        }
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
impl Memory for PPU {
    fn read_byte(&mut self, index: u16) -> u8 {
        match index {
            0x0000..=0x1FFF => {
                self.chr_rom[index as usize]
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
            0x3F00..=0x3F1F => todo!(),
            0x3F20..=0x3FFF => todo!(),
            _ => panic!("Unexpected address")
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ppuaddr_write_ppudata_read() {
        let mut ppu = PPU::new_empty_chr_rom(Mirroring::Horizontal);

        // write 69 to 0x2001
        ppu.vram[1] = 69;
        // write 70 to 0x2002
        ppu.vram[2] = 70;

        ppu.write_ppuaddr(0x20);
        ppu.write_ppuaddr(0x01);

        ppu.read_ppudata();
        assert_eq!(69, ppu.read_ppudata());
        assert_eq!(70, ppu.read_ppudata());
    }

    #[test]
    fn test_ppu_addr_write_ppudata_read_page_cross() {
        let mut ppu = PPU::new_empty_chr_rom(Mirroring::Horizontal);

        ppu.vram[0x01FF] = 69;
        ppu.vram[0x0200] = 70;

        ppu.write_ppuaddr(0x21);
        ppu.write_ppuaddr(0xFF);
        
        ppu.read_ppudata();
        assert_eq!(69, ppu.read_ppudata());
        assert_eq!(70, ppu.read_ppudata());
    }

    #[test]
    fn test_ppu_addr_write_ppu_data_read_inc_by_32() {
        let mut ppu = PPU::new_empty_chr_rom(Mirroring::Horizontal);

        ppu.write_ppuctrl(0b100);
        ppu.vram[0x01FF] = 69;
        ppu.vram[0x01FF + 32] = 70;
        ppu.vram[0x01FF + 64] = 71;

        ppu.write_ppuaddr(0x21);
        ppu.write_ppuaddr(0xFF);

        ppu.read_ppudata();
        assert_eq!(69, ppu.read_ppudata());
        assert_eq!(70, ppu.read_ppudata());
        assert_eq!(71, ppu.read_ppudata());
    }

    #[test]
    fn test_ppuaddr_write_ppudata_write() {
        let mut ppu = PPU::new_empty_chr_rom(Mirroring::Horizontal);

        ppu.write_ppuaddr(0x20);
        ppu.write_ppuaddr(0x01);
        ppu.write_ppudata(69);

        assert_eq!(69, ppu.vram[0x0001])
    }

    #[test]
    fn test_write_oamdma() {
        let mut ppu = PPU::new_empty_chr_rom(Mirroring::Horizontal);

        let mut dma_data = [0u8; 256];
        for i in 0..=255u8 {
            dma_data[i as usize] = i;
        }

        ppu.write_oamaddr(0x10);
        ppu.write_oamdma(&dma_data);

        for i in 0..=255u8 {
            assert_eq!(i, ppu.oam_data[i.wrapping_add(0x10) as usize]);
        }

    }
    #[test]
    fn test_mirror_vram_addr_horizontal() {
        let ppu = PPU::new_empty_chr_rom(Mirroring::Horizontal);
        
        let addr1 = 0x2000 + 0x0100;  // between 0x2000-0x2400
        let addr2 = 0x2400 + 0x0100;  // between 0x2400-0x2800
        let addr3 = 0x2800 + 0x0100;  // between 0x2400-0x2800
        let addr4 = 0x2c00 + 0x0100;  // between 0x2400-0x2800
        assert_eq!(0x0100, ppu.mirror_vram_addr(addr1));
        assert_eq!(0x0100, ppu.mirror_vram_addr(addr2));
        assert_eq!(0x0500, ppu.mirror_vram_addr(addr3));
        assert_eq!(0x0500, ppu.mirror_vram_addr(addr4));
    }

    #[test]
    fn test_mirror_vram_addr_vertical() {
        let ppu = PPU::new_empty_chr_rom(Mirroring::Vertical);
        
        let addr1 = 0x2000 + 0x0100;  // between 0x2000-0x2400
        let addr2 = 0x2400 + 0x0100;  // between 0x2400-0x2800
        let addr3 = 0x2800 + 0x0100;  // between 0x2400-0x2800
        let addr4 = 0x2c00 + 0x0100;  // between 0x2400-0x2800
        assert_eq!(0x0100, ppu.mirror_vram_addr(addr1));
        assert_eq!(0x0500, ppu.mirror_vram_addr(addr2));
        assert_eq!(0x0100, ppu.mirror_vram_addr(addr3));
        assert_eq!(0x0500, ppu.mirror_vram_addr(addr4));
    }

    #[test]
    fn test_mirror_vram_addr_0x3000_to_0x3eff() {
        let mut ppu = PPU::new_empty_chr_rom(Mirroring::Vertical);
        // put dummy data in vram
        for i in 0..0x800 {
            ppu.vram[i] = i as u8;
        }

        for i in 0x2000..0x2EFF {
            let j = i + 0x1000;
            assert_eq!(ppu.read_byte(i), ppu.read_byte(j));
        }
    }

}