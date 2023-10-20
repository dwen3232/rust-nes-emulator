mod cpu_state;
mod cpu_action;
mod cpu_bus;
mod instructions;

pub use cpu_state::{
    CpuState, CpuStatus
};
pub use cpu_bus::CpuBus;
pub use cpu_action::CpuAction;


pub trait CPU {
    // Updates state to after next CPU instruction
    fn next_cpu_instruction(&mut self) -> Result<(), String>;

    fn reset(&mut self);
}