use crate::{cpu::{CpuState, CpuBus}, common::Memory};

use super::{
    Instruction,
    decode_opcode, AddressingMode, Param, Opcode,
};



pub fn parse_instruction(cpu_bus: &mut CpuBus) -> Result<Instruction, String> {
    // Get next three bytes of prg_rom. Instructions are at most 3 bytes in length
    let program_counter = cpu_bus.cpu_state.program_counter as usize;
    let next_bytes = &cpu_bus.rom_state.prg_rom[program_counter..(program_counter + 3)];
    
    // TODO! remove the side effects in the flag logic from this function
    let (opcode, mode, base_cycles) = decode_opcode(next_bytes[0])?;
    let param = decode_param(cpu_bus, &next_bytes[1..], &mode);
    let cycles = base_cycles + compute_extra_cycles(cpu_bus.cpu_state, &opcode, &mode);
    Ok(Instruction {
        opcode, param, cycles
    })
}

pub fn decode_param(cpu_bus: &mut CpuBus, prg_slice: &[u8], mode: &AddressingMode) -> Param {
    // Based on the addressing mode, read `n` number of argument bytes from the program and process it into a parameter
    // to be used by some instruction
    // Returns the number of cycles to read the argument, NOT INCLUDING THE CYCLE TO DECODE THE INSTRUCTION

    match mode {
        AddressingMode::Implicit => {
            Param::None
        },
        AddressingMode::Accumulator => {
            Param::Value(cpu_bus.cpu_state.reg_a)
        },
        AddressingMode::Immediate | AddressingMode::Relative => {
            Param::Value(cpu_bus.read_byte(cpu_bus.cpu_state.program_counter))
        },
        AddressingMode::IndirectJump => {
            // 6502 has a edge case with page boundary when performing indirect jumps
            // AN INDIRECT JUMP MUST NEVER USE A VECTOR BEGINNING ON THE LAST BYTE OF A PAGE
            // http://www.6502.org/tutorials/6502opcodes.html#JMP

            // if address $3000 contains $40, $30FF contains $80, and $3100 contains $50, 
            // the result of JMP ($30FF) will be a transfer of control to $4080 rather than $5080 as you intended 
            // i.e. the 6502 took the low byte of the address from $30FF and the high byte from $3000.

            // first read two bytes
            let mem_addr = cpu_bus.read_two_bytes(cpu_bus.cpu_state.program_counter);

            // read the two bytes from memory and form it into a mem addr
            let mem_addr = if mem_addr & 0x0FF == 0x0FF {
                let lsb = cpu_bus.read_byte(mem_addr) as u16;
                let msb = cpu_bus.read_byte(mem_addr & 0xFF00) as u16;
                (msb << 8) + lsb
            } else {
                cpu_bus.read_two_bytes(mem_addr)
            };
            // IndirectJump does not read the address
            Param::Address(mem_addr)
        },
        AddressingMode::Absolute => {
            let mem_addr = cpu_bus.read_two_bytes(cpu_bus.cpu_state.program_counter);
            Param::Address(mem_addr)
        },
        AddressingMode::AbsoluteJump => {
            let mem_addr = cpu_bus.read_two_bytes(cpu_bus.cpu_state.program_counter);
            // AbsoluteJump does not read the address
            Param::Address(mem_addr)
        },
        AddressingMode::ZeroPage => {
            // read single byte, msb is always 0x00
            let zero_page_addr = cpu_bus.read_byte(cpu_bus.cpu_state.program_counter) as u16;
            Param::Address(zero_page_addr)
        },
        AddressingMode::ZeroPageIndexX => {
            let zero_page_addr = cpu_bus.read_byte(cpu_bus.cpu_state.program_counter).wrapping_add(cpu_bus.cpu_state.reg_x) as u16;
            Param::Address(zero_page_addr)
        },
        AddressingMode::ZeroPageIndexY => {
            let zero_page_addr = cpu_bus.read_byte(cpu_bus.cpu_state.program_counter).wrapping_add(cpu_bus.cpu_state.reg_y) as u16;
            Param::Address(zero_page_addr)
        },
        AddressingMode::AbsoluteIndexX => {
            // Form <instruction> <addr>, X where <addr> is u16, specifies the value of read(<addr> + 1)
            let orig_addr = cpu_bus.read_two_bytes(cpu_bus.cpu_state.program_counter);
            let orig_msb = (orig_addr >> 8) as u8;
            let mem_addr = orig_addr.wrapping_add(cpu_bus.cpu_state.reg_x as u16);
            let msb = (mem_addr >> 8) as u8;
            cpu_bus.cpu_state.page_cross_flag = (orig_msb != msb);
            Param::Address(mem_addr)
        },
        AddressingMode::AbsoluteIndexY => {
            // Same as AbsoluteIndexX, but with reg_y instead
            let orig_addr = cpu_bus.read_two_bytes(cpu_bus.cpu_state.program_counter);
            let orig_msb = (orig_addr >> 8) as u8;
            let mem_addr = orig_addr.wrapping_add(cpu_bus.cpu_state.reg_y as u16);
            let msb = (mem_addr >> 8) as u8;
            cpu_bus.cpu_state.page_cross_flag = (orig_msb != msb);
            Param::Address(mem_addr)
        },
        AddressingMode::IndirectX => {
            // Form <instruction (<addr>, X), where <addr> is u8
            let zero_page_addr = (cpu_bus.read_byte(cpu_bus.cpu_state.program_counter).wrapping_add(cpu_bus.cpu_state.reg_x)) as u16;
            // TODO: may need to re-evaluate how this is done when there's a page cross
            let mem_addr = cpu_bus.read_two_page_bytes(zero_page_addr);
            Param::Address(mem_addr)
        },
        AddressingMode::IndirectY => {
            let zero_page_addr = cpu_bus.read_byte(cpu_bus.cpu_state.program_counter) as u16;
            // TODO: may need to re-evaluate how this is done when there's a page cross
            let orig_addr = cpu_bus.read_two_page_bytes(zero_page_addr);
            let orig_msb = (orig_addr >> 8) as u8;
            let mem_addr = orig_addr.wrapping_add(cpu_bus.cpu_state.reg_y as u16);
            let msb = (mem_addr >> 8) as u8;
            cpu_bus.cpu_state.page_cross_flag = (orig_msb != msb);
            (Param::Address(mem_addr))
        },
    }
    
    
}

fn compute_extra_cycles(cpu_state: &mut CpuState, opcode: &Opcode, addressing_mode: &AddressingMode) -> u8 {
    match (opcode, addressing_mode) {
        (Opcode::ADC | Opcode::AND | Opcode::CMP | Opcode::EOR |Opcode::LDA | 
         Opcode::LDX | Opcode::LDY | Opcode::ORA | Opcode::SBC, 
         AddressingMode::AbsoluteIndexX | AddressingMode::AbsoluteIndexY | AddressingMode::IndirectY) => {
            cpu_state.page_cross_flag as u8
        },
        (Opcode::BPL | Opcode::BMI | Opcode::BVC | Opcode::BVS |
         Opcode::BCC | Opcode::BCS | Opcode::BNE | Opcode::BEQ, _) => {
            (cpu_state.branch_flag as u8) + ((cpu_state.branch_flag & cpu_state.page_cross_flag) as u8)
        },
        _ => 0
    }
}
