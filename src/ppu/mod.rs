mod ppu_state;
mod ppu_action;
mod ppu_bus;

pub use ppu_state::PpuState;
pub use ppu_action::PpuAction;
pub use ppu_bus::PpuBus;

pub trait PPU {
    // Updates state to after next PPU cycle (next frame)
    fn next_ppu_cycle(&mut self) -> Result<(), String>;
}