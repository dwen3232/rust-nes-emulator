use crate::controller::{ControllerState, Controller};
use crate::cpu::{
    CpuAction, CpuState, CPU
};
// use crate::ppu::ppu_state::PpuState;
use crate::ppu::{
    PpuAction, PpuState, PPU
};
use crate::rom::ROM;


// TODO: replace these!
type Program = ();
type ProgramTrace = ();

pub trait NES {
    // pub fn next_cpu_cycle();
    
    // Updates state to after next CPU instruction
    fn next_cpu_instruction(&mut self) -> Result<(), String>;

    // Updates state to after next PPU cycle (next frame)
    fn next_ppu_cycle(&mut self) -> Result<(), String>;
    
    // Loads a program
    fn load_program(&mut self, program: Program) -> Result<(), String>;

    // Resets the console
    fn reset(&mut self) -> Result<(), String>;

    // Look into CPU state
    fn peek_cpu_state(&self) -> CpuState;

    // Look into PPU state
    fn peek_ppu_state(&self) -> PpuState;

    // Prints a program trace that can be used for testing (might want to move this to a NewType)
    fn create_program_trace(&self) -> ProgramTrace;
}

struct ActionNES {
    cpu_state: CpuState,
    ppu_state: PpuState,
    con_state: Controller,
    rom_state: ROM,

    // program_loader:

}

impl NES for ActionNES {
    // Updates state to after next CPU instruction
    fn next_cpu_instruction(&mut self) -> Result<(), String> {
        CpuAction::new(
            &mut self.cpu_state, 
            &mut self.ppu_state, 
            &mut self.con_state, 
            &self.rom_state
        ).next_cpu_instruction()

    }

    // Updates state to after next PPU cycle (next frame)
    fn next_ppu_cycle(&mut self) -> Result<(), String>{
        PpuAction::new(&mut self.ppu_state).next_ppu_cycle()
    }
    
    // Loads a program
    fn load_program(&mut self, program: Program) -> Result<(), String>{
        todo!()
    }

    // Resets the console
    fn reset(&mut self) -> Result<(), String> {
        todo!()
    }

    // Look into CPU state
    fn peek_cpu_state(&self) -> CpuState {
        self.cpu_state
    }

    // Look into PPU state
    fn peek_ppu_state(&self) -> PpuState {
        self.ppu_state
    }

    // Prints a program trace that can be used for testing (might want to move this to a NewType)
    fn create_program_trace(&self) -> ProgramTrace {
        todo!()
    }
}
