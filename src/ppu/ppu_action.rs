use crate::rom::{Mirroring, ROM};

use super::{PpuState, ppu_state::PpuStatus, PpuBus};

pub struct PpuAction<'a, 'b> {
    ppu_state: &'a mut PpuState,
    rom: &'b ROM,
}

impl<'a, 'b> PpuAction<'a, 'b> {
    pub fn new(ppu_state: &'a mut PpuState, rom: &'b ROM) -> Self {
        PpuAction { ppu_state, rom }
    }

    // Blatant violation of SRP, but easiest way to do this atm
    // Return true if on new frame
    pub fn update_ppu_and_check_for_new_frame(&mut self) -> bool {
        if self.ppu_state.cycle_counter < 341 {
            return false;
        }
        if self.is_sprite_zero_hit() {
            // sprite zero hit flag is reset on vblank
            self.ppu_state.ppustatus.set_sprite_zero_hit(true);
        }
        self.ppu_state.cycle_counter = self.ppu_state.cycle_counter - 341;
        self.ppu_state.cur_scanline += 1;

        if self.ppu_state.cur_scanline == 241 {
            self.ppu_state.ppustatus.set_vblank_started(true);
            self.ppu_state.ppustatus.set_sprite_zero_hit(false);
            if self.ppu_state.ppuctrl.is_generate_nmi() {
                self.ppu_state.nmi_interrupt_poll = Some(());
            }
        } else if self.ppu_state.cur_scanline >= 262 {
            self.ppu_state.cur_scanline = 0;
            self.ppu_state.nmi_interrupt_poll = None;
            self.ppu_state.ppustatus.set_vblank_started(false);
            self.ppu_state.ppustatus.set_sprite_zero_hit(false);
            return true;
        }
        return false;
    }

    pub fn write_ppuctrl(&mut self, data: u8) {
        let prev_is_generate_nmi = self.ppu_state.ppuctrl.is_generate_nmi();
        self.ppu_state.ppuctrl.write(data);
        let is_vblank_started = self.ppu_state.ppustatus.is_vblank_started();
        let cur_is_generate_nmi = self.ppu_state.ppuctrl.is_generate_nmi();
        // Set NMI Interrupt signal if PPU is in VBLANK and GENERATE_NMI changes from 0 to 1
        if !prev_is_generate_nmi && cur_is_generate_nmi && is_vblank_started {
            self.ppu_state.nmi_interrupt_poll = Some(())
        }
    }

    pub fn write_ppumask(&mut self, data: u8) {
        self.ppu_state.ppumask.write(data);
    }

    pub fn read_ppustatus(&mut self) -> u8 {
        let bits = self.ppu_state.ppustatus.bits();
        self.ppu_state.ppustatus.remove(PpuStatus::VBLANK_STARTED);
        self.ppu_state.ppuscroll.reset();
        self.ppu_state.ppuaddr.reset();
        bits
    }

    pub fn write_oamaddr(&mut self, data: u8) {
        self.ppu_state.oamaddr.write(data);
    }

    pub fn write_oamdata(&mut self, data: u8) {
        self.ppu_state.oam_data[self.ppu_state.oamaddr.read() as usize] = data;
        self.ppu_state.oamaddr.increment();
    }

    pub fn write_oamdma(&mut self, data: &[u8; 256]) {
        for byte in data.iter() {
            self.ppu_state.oam_data[self.ppu_state.oamaddr.read() as usize] = *byte;
            self.ppu_state.oamaddr.increment();
        }
    }

    pub fn read_oamdata(&self) -> u8 {
        self.ppu_state.oam_data[self.ppu_state.oamaddr.read() as usize]
    }

    pub fn write_ppuscroll(&mut self, data: u8) {
        self.ppu_state.ppuscroll.write(data);
    }

    pub fn write_ppuaddr(&mut self, data: u8) {
        self.ppu_state.ppuaddr.write(data);
    }

    pub fn read_ppudata(&mut self) -> u8 {
        let addr = self.ppu_state.ppuaddr.read();
        // Retrieve previous value in buffer
        let result = self.ppu_state.ppudata;
        // Store in ppudata as buffer
        self.ppu_state.ppudata = self.as_ppu_bus().read_byte(addr);
        // Increment address
        let inc_value = self.ppu_state.ppuctrl.get_vram_addr_inc_value();
        self.ppu_state.ppuaddr.increment(inc_value);
        return result;
    }

    pub fn write_ppudata(&mut self, data: u8) {
        let addr = self.ppu_state.ppuaddr.read();
        self.as_ppu_bus().write_byte(addr, data);
        // Increment address
        let inc_value = self.ppu_state.ppuctrl.get_vram_addr_inc_value();
        self.ppu_state.ppuaddr.increment(inc_value);
    }

    fn as_ppu_bus(&mut self) -> PpuBus {
        PpuBus::new(&mut self.ppu_state, &self.rom)
    }

    fn is_sprite_zero_hit(&self) -> bool {
        let y = self.ppu_state.oam_data[0] as usize;
        let x = self.ppu_state.oam_data[3] as usize;
        // we check <= cycle_counter because ppu is not being simulated tick by tick
        (y ==self.ppu_state.cur_scanline) && (x <= self.ppu_state.cycle_counter) && self.ppu_state.ppumask.is_show_sprites()
    }
}