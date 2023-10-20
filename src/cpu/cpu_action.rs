use crate::ppu::PpuState;
use crate::rom::ROM;
use crate::controller::Controller;

use super::instructions::{
    parse_instruction,
    execute_instruction,
};
use super::{CPU, CpuState, CpuBus};

pub struct CpuAction<'a, 'b, 'c, 'd> {
    cpu_state: &'a mut CpuState,
    ppu_state: &'b mut PpuState,
    con_state: &'c mut Controller,
    rom_state: &'d ROM,
}

impl<'a, 'b, 'c, 'd> CpuAction<'a, 'b, 'c, 'd> {
    pub fn new(
        cpu_state: &'a mut CpuState, 
        ppu_state: &'b mut PpuState,
        con_state: &'c mut Controller,
        rom_state: &'d ROM,
    ) -> Self {
        CpuAction {
            cpu_state, ppu_state, rom_state, con_state
        }
    }
}

impl CPU for CpuAction<'_, '_, '_, '_> {
    fn next_cpu_instruction(&mut self) -> Result<(), String> {
        let instruction = parse_instruction(self.cpu_state, &self.rom_state.prg_rom)?;
        let mut cpu_bus = CpuBus::new(self.cpu_state, &mut self.ppu_state, &self.rom_state, self.con_state);
        execute_instruction(&mut cpu_bus, instruction)?;

        
        Ok(())
    }

    fn reset(&mut self) {
        todo!()
    }
}