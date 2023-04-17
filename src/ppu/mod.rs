use bitflags::bitflags;



// PPUCTRL	$2000	VPHB SINN	NMI enable (V), PPU master/slave (P), sprite height (H), background tile select (B), sprite tile select (S), increment mode (I), nametable select (NN)
// PPUMASK	$2001	BGRs bMmG	color emphasis (BGR), sprite enable (s), background enable (b), sprite left column enable (M), background left column enable (m), greyscale (G)
// PPUSTATUS	$2002	VSO- ----	vblank (V), sprite 0 hit (S), sprite overflow (O); read resets write pair for $2005/$2006
// OAMADDR	$2003	aaaa aaaa	OAM read/write address
// OAMDATA	$2004	dddd dddd	OAM data read/write
// PPUSCROLL	$2005	xxxx xxxx	fine scroll position (two writes: X scroll, Y scroll)
// PPUADDR	$2006	aaaa aaaa	PPU read/write address (two writes: most significant byte, least significant byte)
// PPUDATA	$2007	dddd dddd	PPU data read/write
// OAMDMA	$4014	aaaa aaaa	OAM DMA high address

#[derive(Debug)]
struct PPU {
    ppuctrl: PpuControl,
    ppumask: PpuMask,
    ppustatus:PpuStatus,
}

// pub trait PPU {
//     fn write_to_ctrl(&mut self, value: u8);
//     fn write_to_mask(&mut self, value: u8);
//     fn read_status(&mut self) -> u8; 
//     fn write_to_oam_addr(&mut self, value: u8);
//     fn write_to_oam_data(&mut self, value: u8);
//     fn read_oam_data(&self) -> u8;
//     fn write_to_scroll(&mut self, value: u8);
//     fn write_to_ppu_addr(&mut self, value: u8);
//     fn write_to_data(&mut self, value: u8);
//     fn read_data(&mut self) -> u8;
//     fn write_oam_dma(&mut self, value: &[u8; 256]);
// }
impl PPU {
    fn write_to_ctrl(&mut self, data: u8) {

    }

    fn write_to_mask(&mut self, data: u8) {
        
    }

    fn read_status(&mut self) -> u8 {
        todo!()
    }

    fn wrote_to_oam_addr(&mut self, data: u8) {

    }

    fn write_to_oam_data(&mut self, data: u8) {

    }

    fn read_oam_data(&self) -> u8 {
        todo!()
    }

    
}



bitflags! {
    // PPUCTRL
    // 7  bit  0
    // ---- ----
    // VPHB SINN
    // |||| ||||
    // |||| ||++- Base nametable address
    // |||| ||    (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
    // |||| |+--- VRAM address increment per CPU read/write of PPUDATA
    // |||| |     (0: add 1, going across; 1: add 32, going down)
    // |||| +---- Sprite pattern table address for 8x8 sprites
    // ||||       (0: $0000; 1: $1000; ignored in 8x16 mode)
    // |||+------ Background pattern table address (0: $0000; 1: $1000)
    // ||+------- Sprite size (0: 8x8 pixels; 1: 8x16 pixels – see PPU OAM#Byte 1)
    // |+-------- PPU master/slave select
    // |          (0: read backdrop from EXT pins; 1: output color on EXT pins)
    // +--------- Generate an NMI at the start of the
    //         vertical blanking interval (0: off; 1: on)

    #[derive(Debug, Clone)]
    pub struct PpuControl: u8 {
        const NAMETABLE_0 =             0b0000_0001;
        const NAMETABLE_1 =             0b0000_0010;
        const VRAM_ADDR_INC =           0b0000_0100;
        const SPRITE_PATTERN_ADDR =     0b0000_1000;
        const BACKGROUND_PATTERN_ADDR = 0b0001_0000;
        const SPRITE_SIZE =             0b0010_0000;
        const MASTER_SLAVE_SELECT =     0b0100_0000;
        const GENERATE_NMI =            0b1000_0000;
    }
}

impl PpuControl {
    pub fn get_name_table_addr(&self) -> u16 {
        match self.bits() & 0b11 {
            0b00 => 0x2000,
            0b01 => 0x2400,
            0b10 => 0x2800,
            0b11 => 0x2C00,
            _ => panic!("impossible")
        }
    }

    pub fn get_vram_addr_inc_value(&self) -> u8 {
        if self.contains(PpuControl::VRAM_ADDR_INC) {
            32
        } else {
            1
        }
    }

    pub fn get_sprite_pattern_addr(&self) -> u16 {
        if self.contains(PpuControl::SPRITE_PATTERN_ADDR) {
            0x1000
        } else {
            0
        }
    }

    pub fn get_background_pattern_addr(&self) -> u16 {
        if self.contains(PpuControl::BACKGROUND_PATTERN_ADDR) {
            0x1000
        } else {
            0
        }
    }

    pub fn get_sprite_size(&self) -> (u8, u8) {
        if self.contains(PpuControl::SPRITE_SIZE) {
            (8, 16)
        } else {
            (8, 8)
        }
    }

    pub fn get_master_slave_select(&self) -> bool {
        self.contains(PpuControl::MASTER_SLAVE_SELECT)
    }

    pub fn get_generate_nmi(&self) -> bool {
        self.contains(PpuControl::GENERATE_NMI)
    }

    pub fn write(&mut self, data: u8) {
        // Not sure if this actually works...
        *self = PpuControl::from_bits_truncate(data)
    }
}

bitflags! {
    // 7  bit  0
    // ---- ----
    // BGRs bMmG
    // |||| ||||
    // |||| |||+- Greyscale (0: normal color, 1: produce a greyscale display)
    // |||| ||+-- 1: Show background in leftmost 8 pixels of screen, 0: Hide
    // |||| |+--- 1: Show sprites in leftmost 8 pixels of screen, 0: Hide
    // |||| +---- 1: Show background
    // |||+------ 1: Show sprites
    // ||+------- Emphasize red (green on PAL/Dendy)
    // |+-------- Emphasize green (red on PAL/Dendy)
    // +--------- Emphasize blue
    #[derive(Debug, Clone)]
    pub struct PpuMask: u8 {
        const GREYSCALE =           0b0000_0001;
        const BACKGROUND_LEFTMOST = 0b0000_0010;
        const SPRITES_LEFTMOST =    0b0000_0100;
        const SHOW_BACKGROUND =     0b0000_1000;
        const SHOW_SPRITES =        0b0001_0000;
        const EMPHASIZE_RED =       0b0010_0000;
        const EMPHASIZE_GREEN =     0b0100_0000;
        const EMPHASIZE_BLUE =      0b1000_0000;
    }
}

impl PpuMask {
    pub fn write(&mut self, data: u8) {
        *self = PpuMask::from_bits_truncate(data)
    }
}

bitflags! {
    // 7  bit  0
    // ---- ----
    // VSO. ....
    // |||| ||||
    // |||+-++++- PPU open bus. Returns stale PPU bus contents.
    // ||+------- Sprite overflow. The intent was for this flag to be set
    // ||         whenever more than eight sprites appear on a scanline, but a
    // ||         hardware bug causes the actual behavior to be more complicated
    // ||         and generate false positives as well as false negatives; see
    // ||         PPU sprite evaluation. This flag is set during sprite
    // ||         evaluation and cleared at dot 1 (the second dot) of the
    // ||         pre-render line.
    // |+-------- Sprite 0 Hit.  Set when a nonzero pixel of sprite 0 overlaps
    // |          a nonzero background pixel; cleared at dot 1 of the pre-render
    // |          line.  Used for raster timing.
    // +--------- Vertical blank has started (0: not in vblank; 1: in vblank).
    //         Set at dot 1 of line 241 (the line *after* the post-render
    //         line); cleared after reading $2002 and at dot 1 of the
    //         pre-render line.
    #[derive(Debug, Clone)]
    pub struct PpuStatus: u8 {
        const UNUSED_0 =         0b0000_0001;
        const UNUSED_1 =         0b0000_0010;
        const UNUSED_2 =         0b0000_0100;
        const UNUSED_3 =         0b0000_1000;
        const UNUSED_4 =         0b0001_0000;
        const SPRITE_OVERFLOW =  0b0010_0000;
        const SPRITE_ZERO_HIT =  0b0100_0000;
        const VERT_BLANK_START = 0b1000_0000;
    }
}


#[derive(Debug, Clone)]
struct PpuAddr {
    data: (u8, u8),
    is_set_msb: bool
}

impl PpuAddr {
    pub fn new() -> Self {
        PpuAddr { data: (0, 0), is_set_msb: true}
    }

    pub fn write(&mut self, byte: u8) {
        if self.is_set_msb {
            self.data.1 = byte & 0b0011_1111;
        } else {
            self.data.0 = byte;
        }
        self.is_set_msb = !self.is_set_msb; // flip the bool
    }

    pub fn read(&self) -> u16 { 
        let msb = self.data.1 as u16;
        let lsb = self.data.0 as u16;
        return (msb << 8) + lsb;
    }

    pub fn increment(&mut self, inc: u8) {
        let result = self.read() + (inc as u16);
        self.data.1 = ((result >> 8) & 0b0011_1111) as u8;
        self.data.0 = result as u8;
    }

    pub fn reset(&mut self) {
        self.is_set_msb = true;
    }
    
}

struct PpuData {
    data: u8
}



