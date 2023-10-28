use crate::cpu::{
    CpuBus, CpuStatus, CpuState, cpu_bus, self
};
use crate::common::Memory;
use super::{Opcode, Param, Instruction};




// TODO: maybe make a class called Bus which wraps the CpuBus and the RAM state?
pub fn execute_instruction(cpu_bus: &mut CpuBus, instruction: &Instruction) -> Result<(), String>{
    // FUTURE WORK: can probably condense this more, but not really necessary
    let Instruction{ opcode, param, cycles } = *instruction;
    // TODO: will these instructions ever throw an error?
    match (opcode, param) {
        (Opcode::ADC, Param::Value(val)) => {
            adc(cpu_bus.cpu_state, val)
        },
        (Opcode::ADC, Param::Address(mem_addr)) => {
            let byte = cpu_bus.read_byte(mem_addr);
            adc(cpu_bus.cpu_state, byte)
        },
        (Opcode::AND, Param::Value(val)) => {
            and(cpu_bus.cpu_state, val)
        },
        (Opcode::AND, Param::Address(mem_addr)) => {
            let byte = cpu_bus.read_byte(mem_addr);
            and(cpu_bus.cpu_state, byte)
        },
        (Opcode::ASL, Param::Value(val)) => {
            asl_acc(cpu_bus.cpu_state, val)
        },
        (Opcode::ASL, Param::Address(mem_addr)) => {
            asl(cpu_bus, mem_addr)
        },
        (Opcode::BIT, Param::Value(val)) => {
            bit(cpu_bus.cpu_state, val)
        },
        (Opcode::BIT, Param::Address(mem_addr)) => {
            let byte = cpu_bus.read_byte(mem_addr);
            bit(cpu_bus.cpu_state, byte)
        },
        // BRANCHING
        (Opcode::BPL, Param::Value(val)) => {
            bpl(cpu_bus.cpu_state, val)
        },
        (Opcode::BMI, Param::Value(val)) => {
            bmi(cpu_bus.cpu_state, val)
        },
        (Opcode::BVC, Param::Value(val)) => {
            bvc(cpu_bus.cpu_state, val)
        },
        (Opcode::BVS, Param::Value(val)) => {
            bvs(cpu_bus.cpu_state, val)
        },
        (Opcode::BCC, Param::Value(val)) => {
            bcc(cpu_bus.cpu_state, val)
        },
        (Opcode::BCS, Param::Value(val)) => {
            bcs(cpu_bus.cpu_state, val)
        },
        (Opcode::BNE, Param::Value(val)) => {
            bne(cpu_bus.cpu_state, val)
        },
        (Opcode::BEQ, Param::Value(val)) => {
            beq(cpu_bus.cpu_state, val)
        },
        (Opcode::BRK, Param::None) => {
            brk(cpu_bus.cpu_state) // TODO: remove this, should be an interrupt type
        },
        // COMPARISON
        (Opcode::CMP, Param::Value(val)) => {
            cmp(cpu_bus.cpu_state, val)
        },
        (Opcode::CMP, Param::Address(mem_addr)) => {
            let byte = cpu_bus.read_byte(mem_addr);
            cmp(cpu_bus.cpu_state, byte)

        },
        (Opcode::CPX, Param::Value(val)) => {
            cpx(cpu_bus.cpu_state, val)
        },
        (Opcode::CPX, Param::Address(mem_addr)) => {
            let byte = cpu_bus.read_byte(mem_addr);
            cpx(cpu_bus.cpu_state, byte)
        },
        (Opcode::CPY, Param::Value(val)) => {
            cpy(cpu_bus.cpu_state, val)
        },
        (Opcode::CPY, Param::Address(mem_addr)) => {
            let byte = cpu_bus.read_byte(mem_addr);
            cpy(cpu_bus.cpu_state, byte)
        },
        (Opcode::DEC, Param::Address(mem_addr)) => {
            dec(cpu_bus, mem_addr)
        },
        (Opcode::EOR, Param::Value(val)) => {
            eor(cpu_bus.cpu_state, val)
        },
        (Opcode::EOR, Param::Address(mem_addr)) => {
            let byte = cpu_bus.read_byte(mem_addr);
            eor(cpu_bus.cpu_state, byte)
        },
        (Opcode::CLC, Param::None) => {
            clc(cpu_bus.cpu_state)
        },
        (Opcode::SEC, Param::None) => {
            sec(cpu_bus.cpu_state)
        },
        (Opcode::CLI, Param::None) => {
            cli(cpu_bus.cpu_state)
        },
        (Opcode::SEI, Param::None) => {
            sei(cpu_bus.cpu_state)
        },
        (Opcode::CLV, Param::None) => {
            clv(cpu_bus.cpu_state)
        },
        (Opcode::CLD, Param::None) => {
            cld(cpu_bus.cpu_state)
        },
        (Opcode::SED, Param::None) => {
            sed(cpu_bus.cpu_state)
        },
        (Opcode::INC, Param::Address(mem_addr)) => {
            inc(cpu_bus, mem_addr)
        },
        (Opcode::JMP, Param::Address(mem_addr)) => {
            jmp(cpu_bus.cpu_state, mem_addr)
        },
        (Opcode::JSR, Param::Address(mem_addr)) => {
            jsr(cpu_bus, mem_addr)
        },
        (Opcode::LDA, Param::Value(val)) => {
            lda(cpu_bus.cpu_state, val)
        },
        (Opcode::LDA, Param::Address(mem_addr)) => {
            let byte = cpu_bus.read_byte(mem_addr);
            lda(cpu_bus.cpu_state, byte)
        },
        (Opcode::LDX, Param::Value(val)) => {
            ldx(cpu_bus.cpu_state, val)
        },
        (Opcode::LDX, Param::Address(mem_addr)) => {
            let byte = cpu_bus.read_byte(mem_addr);
            ldx(cpu_bus.cpu_state, byte)
        },
        (Opcode::LDY, Param::Value(val)) => {
            ldy(cpu_bus.cpu_state, val)
        },
        (Opcode::LDY, Param::Address(mem_addr)) => {
            let byte = cpu_bus.read_byte(mem_addr);
            ldy(cpu_bus.cpu_state, byte)
        },
        (Opcode::LSR, Param::Value(val)) => {
            lsr_acc(cpu_bus.cpu_state, val)
        },
        (Opcode::LSR, Param::Address(mem_addr)) => {
            lsr(cpu_bus, mem_addr)
        },
        (Opcode::NOP, Param::None) => {
            todo!()
        },
        (Opcode::ORA, Param::Value(val)) => {
            ora(cpu_bus.cpu_state, val)
        },
        (Opcode::ORA, Param::Address(mem_addr)) => {
            let byte = cpu_bus.read_byte(mem_addr);
            ora(cpu_bus.cpu_state, byte)
        },
        // REGISTER INSTRUCTIONS
        (Opcode::TAX, Param::None) => {
            tax(cpu_bus.cpu_state)
        },
        (Opcode::TXA, Param::None) => {
            txa(cpu_bus.cpu_state)
        },
        (Opcode::DEX, Param::None) => {
            dex(cpu_bus.cpu_state)
        },
        (Opcode::INX, Param::None) => {
            inx(cpu_bus.cpu_state)
        },
        (Opcode::TAY, Param::None) => {
            tay(cpu_bus.cpu_state)
        },
        (Opcode::TYA, Param::None) => {
            tya(cpu_bus.cpu_state)
        },
        (Opcode::DEY, Param::None) => {
            dey(cpu_bus.cpu_state)
        },
        (Opcode::INY, Param::None) => {
            iny(cpu_bus.cpu_state)
        },
        (Opcode::ROL, Param::Value(val)) => {
            rol_acc(cpu_bus.cpu_state, val)
        },
        (Opcode::ROL, Param::Address(mem_addr)) => {
            rol(cpu_bus, mem_addr)
        },
        (Opcode::ROR, Param::Value(val)) => {
            ror_acc(cpu_bus.cpu_state, val)
        },
        (Opcode::ROR, Param::Address(mem_addr)) => {
            ror(cpu_bus, mem_addr)
        },
        (Opcode::RTI, Param::None) => {
            rti(cpu_bus)
        },
        (Opcode::RTS, Param::None) => {
            rts(cpu_bus)
        },
        (Opcode::SBC, Param::Value(val)) => {
            sbc(cpu_bus.cpu_state, val)
        },
        (Opcode::SBC, Param::Address(mem_addr)) => {
            let byte = cpu_bus.read_byte(mem_addr);
            sbc(cpu_bus.cpu_state, byte)
        },
        // STACK INSTRUCTIONS
        (Opcode::TXS, Param::None) => {
            txs(cpu_bus.cpu_state)
        },
        (Opcode::TSX, Param::None) => {
            tsx(cpu_bus.cpu_state)
        },
        (Opcode::PHA, Param::None) => {
            pha(cpu_bus)
        },
        (Opcode::PLA, Param::None) => {
            pla(cpu_bus)
        },
        (Opcode::PHP, Param::None) => {
            php(cpu_bus)
        },
        (Opcode::PLP, Param::None) => {
            plp(cpu_bus)
        },
        (Opcode::STA, Param::Address(mem_addr)) => {
            sta(cpu_bus, mem_addr)
        },
        (Opcode::STX, Param::Address(mem_addr)) => {
            stx(cpu_bus, mem_addr)
        },
        (Opcode::STY, Param::Address(mem_addr)) => {
            sty(cpu_bus, mem_addr)
        }
        _ => panic!("Invalid")
    };
    return Ok(())
}

pub fn adc(cpu_state: &mut CpuState, parameter: u8) {
    /// Affects Flags: N V Z C

    // Cast all relevant values to u16
    let reg_a = cpu_state.reg_a as u16;
    let val = parameter as u16;
    let carry = cpu_state.status.contains(CpuStatus::CARRY) as u16;

    // Add them together
    let sum = reg_a + val + carry;

    // Keep only least significant byte for result
    let result = sum as u8;

    cpu_state.set_negative_flag(result);

    // Check overflow flag; bit 7 must match for operands and result
    if (parameter ^ result) & (cpu_state.reg_a ^ result) & 0b1000_0000 != 0 {
        cpu_state.status.insert(CpuStatus::OVERFLOW);
    } else {
        cpu_state.status.remove(CpuStatus::OVERFLOW);
    }

    cpu_state.set_zero_flag(result);
    cpu_state.set_carry_flag(sum);
    
    // Set accumulator
    cpu_state.reg_a = result;
}

pub fn and(cpu_state: &mut CpuState, parameter: u8) {
    // Affects Flags: N Z
    cpu_state.reg_a = cpu_state.reg_a & parameter;

    cpu_state.set_negative_flag(cpu_state.reg_a);
    cpu_state.set_zero_flag(cpu_state.reg_a);
}

pub fn asl_acc(cpu_state: &mut CpuState, parameter: u8) {
    // Affects Flags: N Z C

    let result = (parameter as u16) << 1;
    cpu_state.reg_a = result as u8;

    cpu_state.set_negative_flag(cpu_state.reg_a);
    cpu_state.set_zero_flag(cpu_state.reg_a);
    cpu_state.set_carry_flag(result);
}

pub fn asl(cpu_bus: &mut CpuBus, address: u16) {
    // Affects Flags: N Z C
    let parameter = cpu_bus.read_byte(address);
    let result = (parameter as u16) << 1;
    cpu_bus.write_byte(address, result as u8);

    cpu_bus.cpu_state.set_negative_flag(result as u8);
    cpu_bus.cpu_state.set_zero_flag(result as u8);
    cpu_bus.cpu_state.set_carry_flag(result);
}

pub fn bit(cpu_state: &mut CpuState, parameter: u8) {
    // Affects Flags: N V Z
    let result = cpu_state.reg_a & parameter;

    cpu_state.set_negative_flag(parameter); // neg if bit 7 in param is 1
    cpu_state.status.set(CpuStatus::OVERFLOW, parameter & 0b0100_0000 != 0); // overflow if bit 6 in param is 1
    cpu_state.set_zero_flag(result);
    
}

// Branching functions
pub fn bpl(cpu_state: &mut CpuState, parameter: u8) {
    cpu_state.branch_flag = !cpu_state.status.contains(CpuStatus::NEGATIVE);
    if cpu_state.branch_flag {
        // we need to left pad parameter with the bit 7 value
        // ex: 11111000 -> 1111111111111000
        let parameter = (parameter as i8) as u16;
        let new_program_counter = cpu_state.program_counter.wrapping_add(parameter);
        cpu_state.page_cross_flag = (new_program_counter >> 8) != (cpu_state.program_counter >> 8);
        cpu_state.program_counter = cpu_state.program_counter.wrapping_add(parameter);
    }
}

pub fn bmi(cpu_state: &mut CpuState, parameter: u8) {
    cpu_state.branch_flag = cpu_state.status.contains(CpuStatus::NEGATIVE);
    if cpu_state.branch_flag {
        // we need to left pad parameter with the bit 7 value
        // ex: 11111000 -> 1111111111111000
        let parameter = (parameter as i8) as u16;
        let new_program_counter = cpu_state.program_counter.wrapping_add(parameter);
        cpu_state.page_cross_flag = (new_program_counter >> 8) != (cpu_state.program_counter >> 8);
        cpu_state.program_counter = cpu_state.program_counter.wrapping_add(parameter);
    }
}

pub fn bvc(cpu_state: &mut CpuState, parameter: u8) {
    cpu_state.branch_flag = !cpu_state.status.contains(CpuStatus::OVERFLOW);
    if cpu_state.branch_flag {
        // we need to left pad parameter with the bit 7 value
        // ex: 11111000 -> 1111111111111000
        let parameter = (parameter as i8) as u16;
        let new_program_counter = cpu_state.program_counter.wrapping_add(parameter);
        cpu_state.page_cross_flag = (new_program_counter >> 8) != (cpu_state.program_counter >> 8);
        cpu_state.program_counter = cpu_state.program_counter.wrapping_add(parameter);
    }
}

pub fn bvs(cpu_state: &mut CpuState, parameter: u8) {
    cpu_state.branch_flag = cpu_state.status.contains(CpuStatus::OVERFLOW);
    if cpu_state.branch_flag {
        // we need to left pad parameter with the bit 7 value
        // ex: 11111000 -> 1111111111111000
        let parameter = (parameter as i8) as u16;
        let new_program_counter = cpu_state.program_counter.wrapping_add(parameter);
        cpu_state.page_cross_flag = (new_program_counter >> 8) != (cpu_state.program_counter >> 8);
        cpu_state.program_counter = cpu_state.program_counter.wrapping_add(parameter);
    }
}

pub fn bcc(cpu_state: &mut CpuState, parameter: u8) {
    cpu_state.branch_flag = !cpu_state.status.contains(CpuStatus::CARRY);
    if cpu_state.branch_flag {
        // we need to left pad parameter with the bit 7 value
        // ex: 11111000 -> 1111111111111000
        let parameter = (parameter as i8) as u16;
        let new_program_counter = cpu_state.program_counter.wrapping_add(parameter);
        cpu_state.page_cross_flag = (new_program_counter >> 8) != (cpu_state.program_counter >> 8);
        cpu_state.program_counter = cpu_state.program_counter.wrapping_add(parameter);
    }
}

pub fn bcs(cpu_state: &mut CpuState, parameter: u8) {
    cpu_state.branch_flag = cpu_state.status.contains(CpuStatus::CARRY);
    if cpu_state.branch_flag {
        // we need to left pad parameter with the bit 7 value
        // ex: 11111000 -> 1111111111111000
        let parameter = (parameter as i8) as u16;
        let new_program_counter = cpu_state.program_counter.wrapping_add(parameter);
        cpu_state.page_cross_flag = (new_program_counter >> 8) != (cpu_state.program_counter >> 8);
        cpu_state.program_counter = cpu_state.program_counter.wrapping_add(parameter);
    }
}

pub fn bne(cpu_state: &mut CpuState, parameter: u8) {
    cpu_state.branch_flag = !cpu_state.status.contains(CpuStatus::ZERO);
    if cpu_state.branch_flag {
        // we need to left pad parameter with the bit 7 value
        // ex: 11111000 -> 1111111111111000
        let parameter = (parameter as i8) as u16;
        let new_program_counter = cpu_state.program_counter.wrapping_add(parameter);
        cpu_state.page_cross_flag = (new_program_counter >> 8) != (cpu_state.program_counter >> 8);
        cpu_state.program_counter = cpu_state.program_counter.wrapping_add(parameter);
    }
}

pub fn beq(cpu_state: &mut CpuState, parameter: u8) {
    cpu_state.branch_flag = cpu_state.status.contains(CpuStatus::ZERO);
    if cpu_state.branch_flag {
        // we need to left pad parameter with the bit 7 value
        // ex: 11111000 -> 1111111111111000
        let parameter = (parameter as i8) as u16;
        let new_program_counter = cpu_state.program_counter.wrapping_add(parameter);
        cpu_state.page_cross_flag = (new_program_counter >> 8) != (cpu_state.program_counter >> 8);
        cpu_state.program_counter = cpu_state.program_counter.wrapping_add(parameter);
    }
}

pub fn brk(cpu_state: &mut CpuState) {
    // BRK causes a non-maskable interrupt and increments the program counter by one TODO figure out what this means
    // Affects Flags: B
    cpu_state.status.insert(CpuStatus::BRK);
}

pub fn cmp(cpu_state: &mut CpuState, parameter: u8) {
    // Affects Flags: N Z C
    let result = cpu_state.reg_a.wrapping_sub(parameter);

    cpu_state.set_negative_flag(result);
    cpu_state.set_zero_flag(result);
    // Special carry flag case
    if cpu_state.reg_a >= parameter {
        cpu_state.status.insert(CpuStatus::CARRY);
    } else {
        cpu_state.status.remove(CpuStatus::CARRY);
    }
}

pub fn cpx(cpu_state: &mut CpuState, parameter: u8) {
    // Affects Flags: N Z C
    let result = cpu_state.reg_x.wrapping_sub(parameter);

    cpu_state.set_negative_flag(result);
    cpu_state.set_zero_flag(result);
    // Special carry flag case
    if cpu_state.reg_x >= parameter {
        cpu_state.status.insert(CpuStatus::CARRY);
    } else {
        cpu_state.status.remove(CpuStatus::CARRY);
    }
}

pub fn cpy(cpu_state: &mut CpuState, parameter: u8) {
    // Affects Flags: N Z C
    let result = cpu_state.reg_y.wrapping_sub(parameter);

    cpu_state.set_negative_flag(result);
    cpu_state.set_zero_flag(result);
    // Special carry flag case
    if cpu_state.reg_y >= parameter {
        cpu_state.status.insert(CpuStatus::CARRY);
    } else {
        cpu_state.status.remove(CpuStatus::CARRY);
    }
}

pub fn dec(cpu_bus: &mut CpuBus, address: u16) {
    // Affects Flags: N Z
    let result = cpu_bus.read_byte(address).wrapping_sub(1);
    cpu_bus.write_byte(address, result);

    cpu_bus.cpu_state.set_negative_flag(result);
    cpu_bus.cpu_state.set_zero_flag(result);
}

pub fn eor(cpu_state: &mut CpuState, parameter: u8) {
    // Affects Flags: N Z
    cpu_state.reg_a = cpu_state.reg_a ^ parameter;

    cpu_state.set_negative_flag(cpu_state.reg_a);
    cpu_state.set_zero_flag(cpu_state.reg_a);
}

// flag instructions
pub fn clc(cpu_state: &mut CpuState) {
    // Clears carry flag
    cpu_state.status.remove(CpuStatus::CARRY);
}

pub fn sec(cpu_state: &mut CpuState) {
    // Sets carry flag
    cpu_state.status.insert(CpuStatus::CARRY);
}

pub fn cli(cpu_state: &mut CpuState) {
    // Clears interrupt flag
    cpu_state.status.remove(CpuStatus::INT_DISABLE);
}

pub fn sei(cpu_state: &mut CpuState) {
    // Sets interrupt flag
    cpu_state.status.insert(CpuStatus::INT_DISABLE);
}

pub fn clv(cpu_state: &mut CpuState) {
    // Clears overflow flag
    cpu_state.status.remove(CpuStatus::OVERFLOW);
}

pub fn cld(cpu_state: &mut CpuState) {
    // Clears decimal flag
    cpu_state.status.remove(CpuStatus::DECIMAL);
}

pub fn sed(cpu_state: &mut CpuState) {
    // Sets decimal flag
    cpu_state.status.insert(CpuStatus::DECIMAL);
}

pub fn inc(cpu_bus: &mut CpuBus, address: u16) {
    // Affects Flags: N Z
    let result = cpu_bus.read_byte(address).wrapping_add(1);
    cpu_bus.write_byte(address, result);

    cpu_bus.cpu_state.set_negative_flag(result);
    cpu_bus.cpu_state.set_zero_flag(result);
}

pub fn jmp(cpu_state: &mut CpuState, address: u16) {
    // Affects Flags: None
    cpu_state.program_counter = address;
}

pub fn jsr(cpu_bus: &mut CpuBus, address: u16) {
    // Affects Flags: None
    let program_counter = cpu_bus.cpu_state.program_counter - 1;
    let lsb = program_counter as u8;
    let msb = (program_counter >> 8) as u8;
    // Push msb first
    cpu_bus.push_to_stack(msb);
    cpu_bus.push_to_stack(lsb);

    cpu_bus.cpu_state.program_counter = address;
}

pub fn lda(cpu_state: &mut CpuState, parameter: u8) {
    // Affects Flags: N Z
    cpu_state.reg_a = parameter;

    cpu_state.set_negative_flag(cpu_state.reg_a);
    cpu_state.set_zero_flag(cpu_state.reg_a);
}

pub fn ldx(cpu_state: &mut CpuState, parameter: u8) {
    // Affects Flags: N Z
    cpu_state.reg_x = parameter;

    cpu_state.set_negative_flag(cpu_state.reg_x);
    cpu_state.set_zero_flag(cpu_state.reg_x);
}

pub fn ldy(cpu_state: &mut CpuState, parameter: u8) {
    // Affects Flags: N Z
    cpu_state.reg_y = parameter;

    cpu_state.set_negative_flag(cpu_state.reg_y);
    cpu_state.set_zero_flag(cpu_state.reg_y);
}

pub fn lsr_acc(cpu_state: &mut CpuState, parameter: u8) {
    // Affects Flags: N Z C
    // LSR for accumulator
    cpu_state.reg_a = parameter >> 1;

    cpu_state.set_negative_flag(cpu_state.reg_a);
    cpu_state.set_zero_flag(cpu_state.reg_a);
    // Special carry flag case
    if parameter % 2 == 1 {
        cpu_state.status.insert(CpuStatus::CARRY);
    } else {
        cpu_state.status.remove(CpuStatus::CARRY);
    }
}

pub fn lsr(cpu_bus: &mut CpuBus, address: u16) {
    // Affects Flags: N Z C
    let parameter = cpu_bus.read_byte(address);
    let result = parameter >> 1;
    cpu_bus.write_byte(address, result);

    cpu_bus.cpu_state.set_negative_flag(result);
    cpu_bus.cpu_state.set_zero_flag(result);
    // Special carry flag case
    if parameter % 2 == 1 {
        cpu_bus.cpu_state.status.insert(CpuStatus::CARRY);
    } else {
        cpu_bus.cpu_state.status.remove(CpuStatus::CARRY);
    }
}

pub fn ora(cpu_state: &mut CpuState, parameter: u8) {
    // Affects Flags: N Z
    cpu_state.reg_a = cpu_state.reg_a | parameter;

    cpu_state.set_negative_flag(cpu_state.reg_a);
    cpu_state.set_zero_flag(cpu_state.reg_a);
}

pub fn tax(cpu_state: &mut CpuState) {
    // Affects Flags: N Z
    cpu_state.reg_x = cpu_state.reg_a;

    cpu_state.set_negative_flag(cpu_state.reg_x);
    cpu_state.set_zero_flag(cpu_state.reg_x);
}

pub fn txa(cpu_state: &mut CpuState) {
    // Affects Flags: N Z
    cpu_state.reg_a = cpu_state.reg_x;

    cpu_state.set_negative_flag(cpu_state.reg_a);
    cpu_state.set_zero_flag(cpu_state.reg_a);
}

pub fn dex(cpu_state: &mut CpuState) {
    // Affects Flags: N Z
    cpu_state.reg_x = cpu_state.reg_x.wrapping_sub(1);

    cpu_state.set_negative_flag(cpu_state.reg_x);
    cpu_state.set_zero_flag(cpu_state.reg_x);
}

pub fn inx(cpu_state: &mut CpuState) {
    // Affects Flags: N Z
    cpu_state.reg_x = cpu_state.reg_x.wrapping_add(1);

    cpu_state.set_negative_flag(cpu_state.reg_x);
    cpu_state.set_zero_flag(cpu_state.reg_x);
}

pub fn tay(cpu_state: &mut CpuState) {
    // Affects Flags: N Z
    cpu_state.reg_y = cpu_state.reg_a;

    cpu_state.set_negative_flag(cpu_state.reg_y);
    cpu_state.set_zero_flag(cpu_state.reg_y);
}

pub fn tya(cpu_state: &mut CpuState) {
    // Affects Flags: N Z
    cpu_state.reg_a = cpu_state.reg_y;

    cpu_state.set_negative_flag(cpu_state.reg_a);
    cpu_state.set_zero_flag(cpu_state.reg_a);
}

pub fn dey(cpu_state: &mut CpuState) {
    // Affects Flags: N Z
    cpu_state.reg_y = cpu_state.reg_y.wrapping_sub(1);

    cpu_state.set_negative_flag(cpu_state.reg_y);
    cpu_state.set_zero_flag(cpu_state.reg_y);
}

pub fn iny(cpu_state: &mut CpuState) {
    // Affects Flags: N Z
    cpu_state.reg_y = cpu_state.reg_y.wrapping_add(1);

    cpu_state.set_negative_flag(cpu_state.reg_y);
    cpu_state.set_zero_flag(cpu_state.reg_y);
}

pub fn rol_acc(cpu_state: &mut CpuState, parameter: u8) {
    // Affects Flags: N Z C
    let mut result = (parameter as u16) << 1;
    if cpu_state.status.contains(CpuStatus::CARRY) {
        result += 1;    // this should be safe from overflow
    }
    cpu_state.reg_a = result as u8;

    cpu_state.set_negative_flag(cpu_state.reg_a);
    cpu_state.set_zero_flag(cpu_state.reg_a);
    cpu_state.set_carry_flag(result);
}

pub fn rol(cpu_bus: &mut CpuBus, address: u16) {
    // Affects Flags: N Z C
    let parameter = cpu_bus.read_byte(address);
    let mut result = (parameter as u16) << 1;
    if cpu_bus.cpu_state.status.contains(CpuStatus::CARRY) {
        result += 1;    // this should be safe from overflow
    }
    cpu_bus.write_byte(address, result as u8);

    cpu_bus.cpu_state.set_negative_flag(result as u8);
    cpu_bus.cpu_state.set_zero_flag(result as u8);
    cpu_bus.cpu_state.set_carry_flag(result);
}

pub fn ror_acc(cpu_state: &mut CpuState, parameter: u8) {
    // Affects Flags: N Z C
    let mut result = parameter >> 1;
    if cpu_state.status.contains(CpuStatus::CARRY) {
        result += 0b1000_0000;
    }
    cpu_state.reg_a = result;
    
    cpu_state.set_negative_flag(result);
    cpu_state.set_zero_flag(result);
    // Special carry flag case
    if parameter % 2 == 1 {
        cpu_state.status.insert(CpuStatus::CARRY);
    } else {
        cpu_state.status.remove(CpuStatus::CARRY);
    }
}

pub fn ror(cpu_bus: &mut CpuBus, address: u16) {
    // Affects Flags: N Z C
    let parameter = cpu_bus.read_byte(address);
    let mut result = parameter >> 1;
    if cpu_bus.cpu_state.status.contains(CpuStatus::CARRY) {
        result += 0b1000_0000;
    }
    cpu_bus.write_byte(address, result);
    
    cpu_bus.cpu_state.set_negative_flag(result);
    cpu_bus.cpu_state.set_zero_flag(result);
    // Special carry flag case
    if parameter % 2 == 1 {
        cpu_bus.cpu_state.status.insert(CpuStatus::CARRY);
    } else {
        cpu_bus.cpu_state.status.remove(CpuStatus::CARRY);
    }
}

pub fn rti(cpu_bus: &mut CpuBus) {
    // Affected Flags: All
    plp(cpu_bus);     // pop status from stack
    let lsb = cpu_bus.pop_from_stack() as u16;
    let msb = cpu_bus.pop_from_stack() as u16;
    cpu_bus.cpu_state.program_counter = (msb << 8) + lsb;
}

pub fn rts(cpu_bus: &mut CpuBus) {
    // Affected Flags: None
    let lsb = cpu_bus.pop_from_stack() as u16;
    let msb = cpu_bus.pop_from_stack() as u16;
    cpu_bus.cpu_state.program_counter = (msb << 8) + lsb + 1;
}

pub fn sbc(cpu_state: &mut CpuState, parameter: u8) {
    // Affects Flags: N V Z C
    // Can just use ADC internally
    adc(cpu_state, parameter ^ 0b1111_1111) // toggle every bit and pass to adc
}   

pub fn txs(cpu_state: &mut CpuState) {
    // Affects Flags: None
    // stack is in the reange 0x100 - 0x1FF
    cpu_state.stack_pointer = cpu_state.reg_x;
}

pub fn tsx(cpu_state: &mut CpuState) {
    // Affects Flags: N Z
    cpu_state.reg_x = cpu_state.stack_pointer;

    cpu_state.set_negative_flag(cpu_state.reg_x);
    cpu_state.set_zero_flag(cpu_state.reg_x);
}

pub fn pha(cpu_bus: &mut CpuBus) {
    // Affects Flags: None
    cpu_bus.push_to_stack(cpu_bus.cpu_state.reg_a);
}

pub fn pla(cpu_bus: &mut CpuBus) {
    // Affects Flags: N Z
    cpu_bus.cpu_state.reg_a = cpu_bus.pop_from_stack();

    cpu_bus.cpu_state.set_negative_flag(cpu_bus.cpu_state.reg_a);
    cpu_bus.cpu_state.set_zero_flag(cpu_bus.cpu_state.reg_a);
}

pub fn php(cpu_bus: &mut CpuBus) {
    // Affects Flags: None
    // Need to push 'status' with BRK set
    // https://www.nesdev.org/wiki/Status_flags#The_B_flag
    let status = cpu_bus.cpu_state.status.clone() | CpuStatus::BRK;
    cpu_bus.push_to_stack(status.bits());
}

pub fn plp(cpu_bus: &mut CpuBus) {
    // Affects Flags: All
    cpu_bus.cpu_state.status = CpuStatus::from_bits(cpu_bus.pop_from_stack()).unwrap();
    // plp discards BRK flag
    // https://www.nesdev.org/wiki/Status_flags#The_B_flag
    cpu_bus.cpu_state.status.remove(CpuStatus::BRK);
    cpu_bus.cpu_state.status.insert(CpuStatus::ALWAYS);
}

pub fn sta(cpu_bus: &mut CpuBus, address: u16) {
    // Affected Flags: None
    cpu_bus.write_byte(address, cpu_bus.cpu_state.reg_a);
}

pub fn stx(cpu_bus: &mut CpuBus, address: u16) {
    // Affected Flags: None
    cpu_bus.write_byte(address, cpu_bus.cpu_state.reg_x);
}

pub fn sty(cpu_bus: &mut CpuBus, address: u16) {
    // Affected Flags: None
    cpu_bus.write_byte(address, cpu_bus.cpu_state.reg_y);
}