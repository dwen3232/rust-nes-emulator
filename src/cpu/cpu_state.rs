use bitflags::bitflags;

use crate::common::Memory;


const STACK_POINTER_INIT: u8 = 0xFD;
const PROGRAM_COUNTER_INIT: u16 = 0x600;

#[derive(Debug, Clone, Copy)]
pub struct CpuState {
    // General purpose registers
    pub reg_a: u8,
    pub reg_x: u8,
    pub reg_y: u8,
    // Special purpose registers
    pub status: CpuStatus,
    pub stack_pointer: u8,
    pub program_counter: u16,

    // Signals (should make this into a bit flag?)
    pub page_cross_flag: bool,
    pub branch_flag: bool,

    cycle_counter: usize,
}

impl CpuState {
    pub fn new() -> Self {
        CpuState {
            reg_a: 0,
            reg_x: 0,
            reg_y: 0,
            // status: CpuStatus::ALWAYS | CpuStatus::BRK,
            status: CpuStatus::ALWAYS | CpuStatus::INT_DISABLE,
            stack_pointer: STACK_POINTER_INIT,      // probably needs to initialize to something else
            program_counter: PROGRAM_COUNTER_INIT,      // same here
            page_cross_flag: false,
            branch_flag: false,
            cycle_counter: 0,
        }
    }

    // TODO: Reset is dependent on the program, which is stored in ROM
    pub fn reset(&mut self) {
        self.reg_a = 0;
        self.reg_x = 0;
        self.reg_y = 0;
        self.stack_pointer = STACK_POINTER_INIT;
        self.status = CpuStatus::ALWAYS | CpuStatus::INT_DISABLE;
        todo!()
        // self.program_counter = self.read_two_bytes(0xFFFC);
        // self.increment_cycle_counter(7);
    }

    pub fn set_carry_flag(&mut self, word: u16) {
        todo!()
    }

    pub fn set_negative_flag(&mut self, byte: u8) {
        todo!()
    }

    pub fn set_zero_flag(&mut self, byte: u8) {
        todo!()
    }
}

impl Memory for CpuState {
    fn read_byte(&mut self, index: u16) -> u8 {
        todo!()
    }

    fn write_byte(&mut self, index: u16, value: u8) {
        todo!()
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct CpuStatus: u8 {
        const CARRY =       0b0000_0001;
        const ZERO =        0b0000_0010;
        const INT_DISABLE = 0b0000_0100;
        const DECIMAL =     0b0000_1000;
        const BRK =         0b0001_0000;
        const ALWAYS =      0b0010_0000;
        const OVERFLOW =    0b0100_0000;
        const NEGATIVE =    0b1000_0000;
    }
}