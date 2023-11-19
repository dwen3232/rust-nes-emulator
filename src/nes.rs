use crate::controller::{Controller, ControllerState};
use crate::cpu::{CpuAction, CpuBus, CpuState, Instruction};
// use crate::ppu::ppu_state::PpuState;
use crate::ppu::{PpuAction, PpuState};
use crate::rom::ROM;
use crate::screen::frame::{Frame};

pub trait NES {
    // pub fn next_cpu_cycle();

    // Updates state to after next CPU instruction
    fn next_cpu_instruction(&mut self) -> Result<Instruction, String>;

    // Updates state to after next PPU cycle (next frame)
    fn next_ppu_frame(&mut self) -> Result<(), String>;

    fn update_controller(&mut self, key: ControllerState, bit: bool);

    // Loads a program
    fn set_rom(&mut self, rom: ROM) -> Result<(), String>;

    fn load_from_path(&mut self, path: &str) -> Result<(), String>;

    // Resets the console
    fn reset(&mut self) -> Result<(), String>;

    // Look into CPU state
    fn peek_cpu_state(&self) -> CpuState;

    // Look into PPU state
    fn peek_ppu_state(&self) -> PpuState;

    // Creates a frame using the current PPU state
    fn render_frame(&self) -> Frame;
}

#[derive(Debug, Default, Clone)]
pub struct ActionNES {
    // TODO: change testing logic so that cpu_state doesn't have to be public!
    pub cpu_state: CpuState,
    pub ppu_state: PpuState,
    pub controller: Controller,
    pub rom: ROM,
}

impl ActionNES {
    pub fn new() -> Self {
        Self::default()
    }

    // TODO: may want to revisit how this is done? Maybe implement From?
    pub fn as_cpu_action(&mut self) -> CpuAction {
        CpuAction::new(
            &mut self.cpu_state,
            &mut self.ppu_state,
            &mut self.controller,
            &self.rom,
        )
    }

    // fn as_ppu_action(&mut self) -> PpuAction {}

    // TODO: change testing logic so that this doesn't have to be public!
    pub fn as_cpu_bus(&mut self) -> CpuBus {
        CpuBus::new(
            &mut self.cpu_state,
            &mut self.ppu_state,
            &mut self.controller,
            &self.rom,
        )
    }

    pub fn as_ppu_action(&mut self) -> PpuAction {
        PpuAction::new(&mut self.ppu_state, &self.rom)
    }
}

impl NES for ActionNES {
    // Updates state to after next CPU instruction
    fn next_cpu_instruction(&mut self) -> Result<Instruction, String> {
        let instruction = self.as_cpu_action().next_cpu_instruction()?;
        Ok(instruction)
    }

    // Updates state to after next PPU cycle (next frame)
    fn next_ppu_frame(&mut self) -> Result<(), String> {
        while {
            let prev_nmi = self.ppu_state.nmi_interrupt_poll.is_some();
            self.as_cpu_action().next_cpu_instruction()?;
            let after_nmi = self.ppu_state.nmi_interrupt_poll.is_some();
            !(!prev_nmi && after_nmi)
        } {}
        Ok(())
    }

    fn update_controller(&mut self, key: ControllerState, bit: bool) {
        self.controller.controller_state.set(key, bit);
    }

    // Loads a program
    fn set_rom(&mut self, rom: ROM) -> Result<(), String> {
        self.rom = rom;
        Ok(())
    }

    fn load_from_path(&mut self, path: &str) -> Result<(), String> {
        self.set_rom(ROM::create_from_nes(path)?)
    }

    // Resets the console
    // TODO: this should trigger some interrupt right?
    fn reset(&mut self) -> Result<(), String> {
        self.cpu_state.reset();
        self.cpu_state.program_counter = self.as_cpu_bus().read_two_bytes(0xFFFC);
        self.cpu_state.cycle_counter += 7;
        self.ppu_state.cycle_counter += 21;
        Ok(())
    }

    // Look into CPU state
    fn peek_cpu_state(&self) -> CpuState {
        self.cpu_state
    }

    // Look into PPU state
    fn peek_ppu_state(&self) -> PpuState {
        self.ppu_state
    }

    // TODO: first few rendered lines are usually invisible, maybe implement that?
    fn render_frame(&self) -> Frame {
        let mut frame = Frame::default();
        frame.render(&self.ppu_state, &self.rom);
        frame
    }
}
