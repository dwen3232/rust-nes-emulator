use super::{PpuState, PPU};

pub struct PpuAction<'a> {
    ppu_state: &'a mut PpuState,
}

impl<'a> PpuAction<'a> {
    pub fn new(ppu_state: &'a mut PpuState) -> Self {
        PpuAction {
            ppu_state
        }
    }
}

impl PPU for PpuAction<'_> {
    fn next_ppu_cycle(&mut self) -> Result<(), String>{
        todo!()
    }
}