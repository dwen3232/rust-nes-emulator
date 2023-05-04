/**
 * https://www.nesdev.org/wiki/CPU_interrupts
 * https://www.nesdev.org/wiki/Status_flags
 * 
 */

pub enum InterruptKind {
    NMI,
    RESET,
    IRQ,
    BRK,
}

// TODO: some of these fields might be unnecessary
pub struct Interrupt {
    pub kind: InterruptKind,
    pub vector: u16,
    pub is_set_b_flag: bool,
    pub is_hardware_interrupt: bool,
}

pub const NMI_INTERRUPT: Interrupt = Interrupt {
    kind: InterruptKind::NMI,
    vector: 0xFFFA,
    is_set_b_flag: false,
    is_hardware_interrupt: true,
};

