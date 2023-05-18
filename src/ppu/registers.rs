use bitflags::bitflags;

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

    pub fn is_master_slave_select(&self) -> bool {
        self.contains(PpuControl::MASTER_SLAVE_SELECT)
    }

    pub fn is_generate_nmi(&self) -> bool {
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

    pub fn is_show_background_leftmost(&self) -> bool {
        self.contains(PpuMask::BACKGROUND_LEFTMOST)
    }

    pub fn is_show_sprites_leftmost(&self) -> bool {
        self.contains(PpuMask::SPRITES_LEFTMOST)
    }

    pub fn is_show_background(&self) -> bool {
        self.contains(PpuMask::SHOW_BACKGROUND)
    }

    pub fn is_show_sprites(&self) -> bool {
        self.contains(PpuMask::SHOW_SPRITES)
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
        const VBLANK_STARTED =   0b1000_0000;
    }
}

impl PpuStatus {
    pub fn set_sprite_overflow(&mut self, status: bool) {
        self.set(PpuStatus::SPRITE_OVERFLOW, status);
    }

    pub fn set_sprite_zero_hit(&mut self, status: bool) {
        self.set(PpuStatus::SPRITE_ZERO_HIT, status);
    }

    pub fn set_vblank_started(&mut self, status: bool) {
        self.set(PpuStatus::VBLANK_STARTED, status);
    }

    pub fn is_vblank_started(&self) -> bool {
        self.contains(PpuStatus::VBLANK_STARTED)
    }
}

#[derive(Debug, Clone)]
pub struct OamAddr {
    data: u8
}

impl OamAddr {
    pub fn new() -> Self {
        OamAddr { data: 0 }
    }
    pub fn read(&self) -> u8 {
        self.data
    }

    pub fn write(&mut self, data: u8) {
        self.data = data;
    }

    pub fn increment(&mut self) {
        // TODO: check this is correct
        self.data = self.data.wrapping_add(1);
    }

}

#[derive(Debug, Clone)]
pub struct OamData {

}

#[derive(Debug, Clone)]
pub struct PpuScroll {
    cam_position_x: u8,
    cam_position_y: u8,
    is_set_position_x: bool
}


// Horizontal offsets range from 0 to 255. "Normal" vertical offsets range from 0 to 239, while values of 240 to 255 are treated as -16 through -1 in a way, but tile data is incorrectly fetched from the attribute table.
// Implies that reading from this is different
// TODO: check this
impl PpuScroll {
    pub fn new() -> Self {
        PpuScroll { cam_position_x: 0, cam_position_y: 0, is_set_position_x: true}
    }

    pub fn write(&mut self, byte: u8) {
        if self.is_set_position_x {
            self.cam_position_x = byte;
        } else {
            self.cam_position_y = byte;
        }
        self.is_set_position_x = !self.is_set_position_x; // flip the bool
    }

    pub fn read(&self) -> (u8, u8) { 
        // Returns (cam_position_x, cam_position_y)
        todo!()
    }

    pub fn reset(&mut self) {
        self.is_set_position_x = true;
    }
    
}



#[derive(Debug, Clone)]
pub struct PpuAddr {
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

#[cfg(test)]
mod tests {
    use bitflags::BitFlags;

    use super::*;

    #[test]
    fn test_write_ppuctrl() {
        let mut ctrl = PpuControl::from_bits_retain(0);
        ctrl.write(0b0000_0011);
        assert_eq!(0b0000_0011, ctrl.bits());
    }
}