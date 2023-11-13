use bitflags::bitflags;


const STACK_POINTER_INIT: u8 = 0xFD;
const PROGRAM_COUNTER_INIT: u16 = 0x600;

// ! This struct should never create a Bus or an Action
#[derive(Debug, Clone, Copy)]
pub struct CpuState {
    // 2KB RAM
    pub ram: [u8; 0x800],
    // General purpose registers
    pub reg_a: u8,
    pub reg_x: u8,
    pub reg_y: u8,
    // Special purpose registers
    pub status: CpuStatus,
    pub stack_pointer: u8,
    pub program_counter: u16,

    // Flags (should make this into a bit flag?)
    pub page_cross_flag: bool,
    pub branch_flag: bool,

    // Interrupts
    pub irq_interrupt_poll: Option<()>,

    pub cycle_counter: usize,
}

impl CpuState {
    pub fn new() -> Self {
        CpuState {
            ram: [0; 0x800],
            reg_a: 0,
            reg_x: 0,
            reg_y: 0,
            // status: CpuStatus::ALWAYS | CpuStatus::BRK,
            status: CpuStatus::ALWAYS | CpuStatus::INT_DISABLE,
            stack_pointer: STACK_POINTER_INIT,      // probably needs to initialize to something else
            program_counter: PROGRAM_COUNTER_INIT,      // same here
            page_cross_flag: false,
            branch_flag: false,
            irq_interrupt_poll: None,
            cycle_counter: 0,
        }
    }

    // TODO: should this reset the rest of the state as well?
    pub fn reset(&mut self) {
        
        self.reg_a = 0;
        self.reg_x = 0;
        self.reg_y = 0;
        self.stack_pointer = STACK_POINTER_INIT;
        // self.status = CpuStatus::ALWAYS | CpuStatus::BRK;
        self.status = CpuStatus::ALWAYS | CpuStatus::INT_DISABLE;

        // self.ram = [0; 0x800];
        // self.program_counter = PROGRAM_COUNTER_INIT;
        // self.page_cross_flag = false;
        // self.branch_flag = false;
        // self.cycle_counter = 0;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialization() {
        let cpu_state = CpuState::new();
        assert_eq!(0, cpu_state.reg_a)
    }
}