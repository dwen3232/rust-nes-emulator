use crate::controller::{ControllerState, Controller};
use crate::cpu::{
    CpuAction, CpuState, Instruction, CpuBus, InstructionMetaData, AddressingMode, Param
};
// use crate::ppu::ppu_state::PpuState;
use crate::ppu::{PpuState, PpuAction};
use crate::rom::ROM;




pub trait NES {
    // pub fn next_cpu_cycle();
    
    // Updates state to after next CPU instruction
    fn next_cpu_instruction(&mut self) -> Result<Instruction, String>;

    // Updates state to after next PPU cycle (next frame)
    fn next_ppu_frame(&mut self) -> Result<Option<()>, String>;
    
    // Loads a program
    fn set_rom(&mut self, rom: ROM) -> Result<(), String>;

    fn load_from_path(&mut self, path: &str) -> Result<(), String>;

    // Resets the console
    fn reset(&mut self) -> Result<(), String>;

    // Look into CPU state
    fn peek_cpu_state(&self) -> CpuState;

    // Look into PPU state
    fn peek_ppu_state(&self) -> PpuState;
}

#[derive(Debug, Clone)]
pub struct ActionNES {
    // TODO: change testing logic so that cpu_state doesn't have to be public!
    pub cpu_state: CpuState,
    pub ppu_state: PpuState,
    pub controller: Controller,
    pub rom: ROM,
}

impl ActionNES {
    pub fn new() -> Self {
        println!("test");
        ActionNES { 
            cpu_state: CpuState::new(), 
            ppu_state: PpuState::new(), 
            controller: Controller::new(), 
            rom: ROM::new(),
        }
    }
    // TODO: may want to revisit how this is done? Maybe implement From?
    fn as_cpu_action(&mut self) -> CpuAction {
        CpuAction::new(&mut self.cpu_state, &mut self.ppu_state, &mut self.controller, &self.rom)
    }

    // fn as_ppu_action(&mut self) -> PpuAction {}

    // TODO: change testing logic so that this doesn't have to be public!
    pub fn as_cpu_bus(&mut self) -> CpuBus {
        CpuBus::new(&mut self.cpu_state, &mut self.ppu_state, &mut self.controller, &self.rom)
    }

    pub fn as_ppu_action(&mut self) -> PpuAction {
        PpuAction::new(&mut self.ppu_state)
    }
    
        
}

impl NES for ActionNES {
    // Updates state to after next CPU instruction
    fn next_cpu_instruction(&mut self) -> Result<Instruction, String> {
        let instruction = self.as_cpu_action().next_cpu_instruction()?;
        self.as_ppu_action().update_ppu_and_check_for_new_frame();
        Ok(instruction)
    }

    // Updates state to after next PPU cycle (next frame)
    fn next_ppu_frame(&mut self) -> Result<Option<()>, String>{
        // TODO: need to run CPU instructions until we're at the next frame
        todo!()
    }
    
    // Loads a program
    fn set_rom(&mut self, rom: ROM) -> Result<(), String>{
        self.rom = rom;
        Ok(())
    }

    fn load_from_path(&mut self, path: &str) -> Result<(), String> {
        self.set_rom(ROM::create_from_nes(path)?)
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
    
}

