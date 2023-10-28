use crate::ppu::PpuState;
use crate::rom::ROM;
use crate::controller::Controller;

use super::instructions::{
    parse_instruction,
    execute_instruction, Instruction,
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
    fn next_cpu_instruction(&mut self) -> Result<Instruction, String> {
        // parse_instruction has side effects
        // TODO: this actually requires CPU bus, since the Param needs to read from memory if it's indirect
        let mut cpu_bus = CpuBus::new(self.cpu_state, &mut self.ppu_state, &self.rom_state, self.con_state);
        let instruction = parse_instruction(&mut cpu_bus)?;

        // create bus and execute instruction with it
        
        execute_instruction(&mut cpu_bus, &instruction)?;

        
        Ok(instruction)
    }

    fn reset(&mut self) {
        todo!()
    }
}