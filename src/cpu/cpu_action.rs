use crate::{controller::Controller, ppu::PpuState, rom::ROM};

use super::instructions::decode_opcode;
use super::{
    instructions::{AddressingMode, InstructionMetaData, Opcode, Param},
    interrupt::{Interrupt, NMI_INTERRUPT},
    CpuBus, CpuState, CpuStatus, Instruction,
};

pub struct CpuAction<'a, 'b, 'c, 'd> {
    cpu_state: &'a mut CpuState,
    ppu_state: &'b mut PpuState,
    controller: &'c mut Controller,
    rom: &'d ROM,
}

impl<'a, 'b, 'c, 'd> CpuAction<'a, 'b, 'c, 'd> {
    pub fn new(
        cpu_state: &'a mut CpuState,
        ppu_state: &'b mut PpuState,
        controller: &'c mut Controller,
        rom: &'d ROM,
    ) -> Self {
        CpuAction {
            cpu_state,
            ppu_state,
            controller,
            rom,
        }
    }

    pub fn next_cpu_instruction(&mut self) -> Result<Instruction, String> {
        // ! TODO: eventually, I want this to follow a pipelining pattern (fetch, decode, execute, mem, wb) or something similar
        // 1. Check for interrupt
        if let Some(()) = self.ppu_state.nmi_interrupt_poll.take() {
            self.execute_interrupt(NMI_INTERRUPT);
        }

        // 2. Read opcode and decode it to an instruction, always takes 1 cycle
        let start_pc = self.cpu_state.program_counter;
        let raw_opcode = self.as_bus().read_byte_from_pc();
        let (opcode, mode, base_cycles) = decode_opcode(raw_opcode)?;

        // 3. Read some number of bytes depending on what the addressing mode is and decode the instruction parameter, may take many cycles
        // Ref: http://www.6502.org/tutorials/6502opcodes.html
        let param = self.read_arg(&mode);
        let end_pc = self.cpu_state.program_counter;
        let length = end_pc - start_pc;

        // 4. Execute the instruction
        self.execute_instruction(&opcode, param)?;

        // 5. Update cycles
        let cycles = base_cycles + self.compute_extra_cycles(&opcode, &mode);
        self.increment_cycle_counters(cycles);

        let meta = InstructionMetaData {
            cycles,
            mode,
            raw_opcode,
            length,
        };
        let instruction = Instruction {
            opcode,
            param,
            meta,
        };
        Ok(instruction)
    }
}

impl<'a, 'b, 'c, 'd> CpuAction<'a, 'b, 'c, 'd> {
    fn as_bus(&mut self) -> CpuBus {
        let Self {
            cpu_state,
            ppu_state,
            controller,
            rom,
        } = self;
        CpuBus::new(cpu_state, ppu_state, controller, rom)
    }

    fn increment_cycle_counters(&mut self, cycles: u8) {
        self.cpu_state.cycle_counter += cycles as usize;
        self.ppu_state.cycle_counter += 3 * cycles as usize;
    }

    fn push_to_stack(&mut self, value: u8) {
        // Stack located from 0x100 to 0x1FF, growing downward
        // For push, need to write first, then decrement
        let stack_addr = 0x100 + (self.cpu_state.stack_pointer as u16);
        self.cpu_state.stack_pointer = self.cpu_state.stack_pointer.wrapping_sub(1);
        self.as_bus().write_byte(stack_addr, value)
    }

    fn pop_from_stack(&mut self) -> u8 {
        // For pop, need to increment first, then read
        self.cpu_state.stack_pointer = self.cpu_state.stack_pointer.wrapping_add(1);
        let stack_addr = 0x100 + (self.cpu_state.stack_pointer as u16);
        self.as_bus().read_byte(stack_addr)
    }

    fn set_zero_flag(&mut self, result: u8) {
        if result == 0 {
            self.cpu_state.status.insert(CpuStatus::ZERO);
        } else {
            self.cpu_state.status.remove(CpuStatus::ZERO);
        }
    }

    fn set_negative_flag(&mut self, result: u8) {
        if result & 0b1000_0000 != 0 {
            self.cpu_state.status.insert(CpuStatus::NEGATIVE);
        } else {
            self.cpu_state.status.remove(CpuStatus::NEGATIVE);
        }
    }

    fn set_carry_flag(&mut self, result: u16) {
        // Check carry flag
        if result > 0xFF {
            self.cpu_state.status.insert(CpuStatus::CARRY);
        } else {
            self.cpu_state.status.remove(CpuStatus::CARRY);
        }
    }
    fn execute_interrupt(&mut self, interrupt: Interrupt) {
        // TODO: I think how interrupts are handled needs to be revisited eventually
        let lsb = self.cpu_state.program_counter as u8;
        let msb = (self.cpu_state.program_counter >> 8) as u8;
        let mut status = self.cpu_state.status;
        // Push BRK flag depending on interrupt type
        status.set(CpuStatus::BRK, interrupt.is_set_b_flag);

        self.push_to_stack(msb);
        self.push_to_stack(lsb);
        self.push_to_stack(status.bits());

        // Set INT_DISABLE flag depending on interrupt type
        self.cpu_state
            .status
            .set(CpuStatus::INT_DISABLE, interrupt.is_hardware_interrupt);
        self.cpu_state.program_counter = self.as_bus().read_two_bytes(interrupt.vector);
    }

    fn compute_extra_cycles(&self, opcode: &Opcode, addressing_mode: &AddressingMode) -> u8 {
        match (opcode, addressing_mode) {
            (
                Opcode::ADC
                | Opcode::AND
                | Opcode::CMP
                | Opcode::EOR
                | Opcode::LDA
                | Opcode::LDX
                | Opcode::LDY
                | Opcode::ORA
                | Opcode::SBC,
                AddressingMode::AbsoluteIndexX
                | AddressingMode::AbsoluteIndexY
                | AddressingMode::IndirectY,
            ) => self.cpu_state.page_cross_flag as u8,
            (
                Opcode::BPL
                | Opcode::BMI
                | Opcode::BVC
                | Opcode::BVS
                | Opcode::BCC
                | Opcode::BCS
                | Opcode::BNE
                | Opcode::BEQ,
                _,
            ) => {
                (self.cpu_state.branch_flag as u8)
                    + ((self.cpu_state.branch_flag & self.cpu_state.page_cross_flag) as u8)
            }
            _ => 0,
        }
    }
}

impl<'a, 'b, 'c, 'd> CpuAction<'a, 'b, 'c, 'd> {
    /// Based on the addressing mode, read `n` number of argument bytes from the program and process it into a parameter
    /// to be used by some instruction
    /// Returns the number of cycles to read the argument, NOT INCLUDING THE CYCLE TO DECODE THE INSTRUCTION
    /// ! Has side effects from page cross and maybe reading using the bus?
    // TODO: want to return (Param, &[u8]) at some point
    fn read_arg(&mut self, mode: &AddressingMode) -> Param {
        // TODO?: I had to create bus in a couple weird places to get this to work, revisit to see if there's a better way to do this
        let mut bus = CpuBus::new(self.cpu_state, self.ppu_state, self.controller, self.rom);
        match mode {
            AddressingMode::Implicit => Param::None,
            AddressingMode::Accumulator => Param::Value(self.cpu_state.reg_a),
            AddressingMode::Immediate | AddressingMode::Relative => {
                Param::Value(bus.read_byte_from_pc())
            }
            AddressingMode::IndirectJump => {
                // 6502 has a edge case with page boundary when performing indirect jumps
                // AN INDIRECT JUMP MUST NEVER USE A VECTOR BEGINNING ON THE LAST BYTE OF A PAGE
                // http://www.6502.org/tutorials/6502opcodes.html#JMP

                // if address $3000 contains $40, $30FF contains $80, and $3100 contains $50,
                // the result of JMP ($30FF) will be a transfer of control to $4080 rather than $5080 as you intended
                // i.e. the 6502 took the low byte of the address from $30FF and the high byte from $3000.

                // first read two bytes
                let mem_addr = bus.read_two_bytes_from_pc();

                // read the two bytes from memory and form it into a mem addr
                let mem_addr = if mem_addr & 0x0FF == 0x0FF {
                    let lsb = bus.read_byte(mem_addr) as u16;
                    let msb = bus.read_byte(mem_addr & 0xFF00) as u16;
                    (msb << 8) + lsb
                } else {
                    bus.read_two_bytes(mem_addr)
                };
                // IndirectJump does not read the address
                Param::Address(mem_addr)
            }
            AddressingMode::Absolute => {
                let mem_addr = bus.read_two_bytes_from_pc();
                Param::Address(mem_addr)
            }
            AddressingMode::AbsoluteJump => {
                let mem_addr = bus.read_two_bytes_from_pc();
                // AbsoluteJump does not read the address
                Param::Address(mem_addr)
            }
            AddressingMode::ZeroPage => {
                // read single byte, msb is always 0x00
                let zero_page_addr = bus.read_byte_from_pc() as u16;
                Param::Address(zero_page_addr)
            }
            AddressingMode::ZeroPageIndexX => {
                let zero_page_addr =
                    bus.read_byte_from_pc().wrapping_add(self.cpu_state.reg_x) as u16;
                Param::Address(zero_page_addr)
            }
            AddressingMode::ZeroPageIndexY => {
                let zero_page_addr =
                    bus.read_byte_from_pc().wrapping_add(self.cpu_state.reg_y) as u16;
                Param::Address(zero_page_addr)
            }
            AddressingMode::AbsoluteIndexX => {
                // Form <instruction> <addr>, X where <addr> is u16, specifies the value of read(<addr> + 1)
                let orig_addr = bus.read_two_bytes_from_pc();
                let orig_msb = (orig_addr >> 8) as u8;
                let mem_addr = orig_addr.wrapping_add(self.cpu_state.reg_x as u16);
                let msb = (mem_addr >> 8) as u8;
                self.cpu_state.page_cross_flag = orig_msb != msb;
                Param::Address(mem_addr)
            }
            AddressingMode::AbsoluteIndexY => {
                // Same as AbsoluteIndexX, but with reg_y instead
                let orig_addr = bus.read_two_bytes_from_pc();
                let orig_msb = (orig_addr >> 8) as u8;
                let mem_addr = orig_addr.wrapping_add(self.cpu_state.reg_y as u16);
                let msb = (mem_addr >> 8) as u8;
                self.cpu_state.page_cross_flag = orig_msb != msb;
                Param::Address(mem_addr)
            }
            AddressingMode::IndirectX => {
                // Form <instruction (<addr>, X), where <addr> is u8
                let base = bus.read_byte_from_pc();
                let zero_page_addr = (base.wrapping_add(self.cpu_state.reg_x)) as u16;
                let mut bus =
                    CpuBus::new(self.cpu_state, self.ppu_state, self.controller, self.rom);
                // TODO: may need to re-evaluate how this is done when there's a page cross
                let mem_addr = bus.read_two_page_bytes(zero_page_addr);
                Param::Address(mem_addr)
            }
            AddressingMode::IndirectY => {
                let zero_page_addr = bus.read_byte_from_pc() as u16;
                // TODO: may need to re-evaluate how this is done when there's a page cross
                let orig_addr = bus.read_two_page_bytes(zero_page_addr);
                let orig_msb = (orig_addr >> 8) as u8;
                let mem_addr = orig_addr.wrapping_add(self.cpu_state.reg_y as u16);
                let msb = (mem_addr >> 8) as u8;
                self.cpu_state.page_cross_flag = orig_msb != msb;
                Param::Address(mem_addr)
            }
        }
    }
}

impl<'a, 'b, 'c, 'd> CpuAction<'a, 'b, 'c, 'd> {
    // TODO: this should borrow parameter
    fn execute_instruction(
        &mut self,
        instruction: &Opcode,
        parameter: Param,
    ) -> Result<(), String> {
        // FUTURE WORK: can probably condense this more, but not really necessary
        match (instruction, parameter) {
            (Opcode::ADC, Param::Value(val)) => self.adc(val),
            (Opcode::ADC, Param::Address(mem_addr)) => {
                let byte = self.as_bus().read_byte(mem_addr);
                self.adc(byte)
            }
            (Opcode::AND, Param::Value(val)) => self.and(val),
            (Opcode::AND, Param::Address(mem_addr)) => {
                let byte = self.as_bus().read_byte(mem_addr);
                self.and(byte)
            }
            (Opcode::ASL, Param::Value(val)) => self.asl_acc(val),
            (Opcode::ASL, Param::Address(mem_addr)) => self.asl(mem_addr),
            (Opcode::BIT, Param::Value(val)) => self.bit(val),
            (Opcode::BIT, Param::Address(mem_addr)) => {
                let byte = self.as_bus().read_byte(mem_addr);
                self.bit(byte)
            }
            // BRANCHING
            (Opcode::BPL, Param::Value(val)) => self.bpl(val),
            (Opcode::BMI, Param::Value(val)) => self.bmi(val),
            (Opcode::BVC, Param::Value(val)) => self.bvc(val),
            (Opcode::BVS, Param::Value(val)) => self.bvs(val),
            (Opcode::BCC, Param::Value(val)) => self.bcc(val),
            (Opcode::BCS, Param::Value(val)) => self.bcs(val),
            (Opcode::BNE, Param::Value(val)) => self.bne(val),
            (Opcode::BEQ, Param::Value(val)) => self.beq(val),
            (Opcode::BRK, Param::None) => {
                self.brk() // TODO: remove this, should be an interrupt type
            }
            // COMPARISON
            (Opcode::CMP, Param::Value(val)) => self.cmp(val),
            (Opcode::CMP, Param::Address(mem_addr)) => {
                let byte = self.as_bus().read_byte(mem_addr);
                self.cmp(byte)
            }
            (Opcode::CPX, Param::Value(val)) => self.cpx(val),
            (Opcode::CPX, Param::Address(mem_addr)) => {
                let byte = self.as_bus().read_byte(mem_addr);
                self.cpx(byte)
            }
            (Opcode::CPY, Param::Value(val)) => self.cpy(val),
            (Opcode::CPY, Param::Address(mem_addr)) => {
                let byte = self.as_bus().read_byte(mem_addr);
                self.cpy(byte)
            }
            (Opcode::DEC, Param::Address(mem_addr)) => self.dec(mem_addr),
            (Opcode::EOR, Param::Value(val)) => self.eor(val),
            (Opcode::EOR, Param::Address(mem_addr)) => {
                let byte = self.as_bus().read_byte(mem_addr);
                self.eor(byte)
            }
            (Opcode::CLC, Param::None) => self.clc(),
            (Opcode::SEC, Param::None) => self.sec(),
            (Opcode::CLI, Param::None) => self.cli(),
            (Opcode::SEI, Param::None) => self.sei(),
            (Opcode::CLV, Param::None) => self.clv(),
            (Opcode::CLD, Param::None) => self.cld(),
            (Opcode::SED, Param::None) => self.sed(),
            (Opcode::INC, Param::Address(mem_addr)) => self.inc(mem_addr),
            (Opcode::JMP, Param::Address(mem_addr)) => self.jmp(mem_addr),
            (Opcode::JSR, Param::Address(mem_addr)) => self.jsr(mem_addr),
            (Opcode::LDA, Param::Value(val)) => self.lda(val),
            (Opcode::LDA, Param::Address(mem_addr)) => {
                let byte = self.as_bus().read_byte(mem_addr);
                self.lda(byte)
            }
            (Opcode::LDX, Param::Value(val)) => self.ldx(val),
            (Opcode::LDX, Param::Address(mem_addr)) => {
                let byte = self.as_bus().read_byte(mem_addr);
                self.ldx(byte)
            }
            (Opcode::LDY, Param::Value(val)) => self.ldy(val),
            (Opcode::LDY, Param::Address(mem_addr)) => {
                let byte = self.as_bus().read_byte(mem_addr);
                self.ldy(byte)
            }
            (Opcode::LSR, Param::Value(val)) => self.lsr_acc(val),
            (Opcode::LSR, Param::Address(mem_addr)) => self.lsr(mem_addr),
            (Opcode::NOP, Param::None) => {
                // TODO: implement this?
            }
            (Opcode::ORA, Param::Value(val)) => self.ora(val),
            (Opcode::ORA, Param::Address(mem_addr)) => {
                let byte = self.as_bus().read_byte(mem_addr);
                self.ora(byte)
            }
            // REGISTER INSTRUCTIONS
            (Opcode::TAX, Param::None) => self.tax(),
            (Opcode::TXA, Param::None) => self.txa(),
            (Opcode::DEX, Param::None) => self.dex(),
            (Opcode::INX, Param::None) => self.inx(),
            (Opcode::TAY, Param::None) => self.tay(),
            (Opcode::TYA, Param::None) => self.tya(),
            (Opcode::DEY, Param::None) => self.dey(),
            (Opcode::INY, Param::None) => self.iny(),
            (Opcode::ROL, Param::Value(val)) => self.rol_acc(val),
            (Opcode::ROL, Param::Address(mem_addr)) => self.rol(mem_addr),
            (Opcode::ROR, Param::Value(val)) => self.ror_acc(val),
            (Opcode::ROR, Param::Address(mem_addr)) => self.ror(mem_addr),
            (Opcode::RTI, Param::None) => self.rti(),
            (Opcode::RTS, Param::None) => self.rts(),
            (Opcode::SBC, Param::Value(val)) => self.sbc(val),
            (Opcode::SBC, Param::Address(mem_addr)) => {
                let byte = self.as_bus().read_byte(mem_addr);
                self.sbc(byte)
            }
            // STACK INSTRUCTIONS
            (Opcode::TXS, Param::None) => self.txs(),
            (Opcode::TSX, Param::None) => self.tsx(),
            (Opcode::PHA, Param::None) => self.pha(),
            (Opcode::PLA, Param::None) => self.pla(),
            (Opcode::PHP, Param::None) => self.php(),
            (Opcode::PLP, Param::None) => self.plp(),
            (Opcode::STA, Param::Address(mem_addr)) => self.sta(mem_addr),
            (Opcode::STX, Param::Address(mem_addr)) => self.stx(mem_addr),
            (Opcode::STY, Param::Address(mem_addr)) => self.sty(mem_addr),
            _ => return Err(String::from("Invalid")),
        };
        Ok(())
    }
}

impl<'a, 'b, 'c, 'd> CpuAction<'a, 'b, 'c, 'd> {
    fn adc(&mut self, parameter: u8) {
        // Affects Flags: N V Z C

        // Cast all relevant values to u16
        let reg_a = self.cpu_state.reg_a as u16;
        let val = parameter as u16;
        let carry = self.cpu_state.status.contains(CpuStatus::CARRY) as u16;

        // Add them together
        let sum = reg_a + val + carry;

        // Keep only least significant byte for result
        let result = sum as u8;

        self.set_negative_flag(result);

        // Check overflow flag; bit 7 must match for operands and result
        if (parameter ^ result) & (self.cpu_state.reg_a ^ result) & 0b1000_0000 != 0 {
            self.cpu_state.status.insert(CpuStatus::OVERFLOW);
        } else {
            self.cpu_state.status.remove(CpuStatus::OVERFLOW);
        }

        self.set_zero_flag(result);
        self.set_carry_flag(sum);

        // Set accumulator
        self.cpu_state.reg_a = result;
    }

    fn and(&mut self, parameter: u8) {
        // Affects Flags: N Z
        self.cpu_state.reg_a &= parameter;

        self.set_negative_flag(self.cpu_state.reg_a);
        self.set_zero_flag(self.cpu_state.reg_a);
    }

    fn asl_acc(&mut self, parameter: u8) {
        // Affects Flags: N Z C

        let result = (parameter as u16) << 1;
        self.cpu_state.reg_a = result as u8;

        self.set_negative_flag(self.cpu_state.reg_a);
        self.set_zero_flag(self.cpu_state.reg_a);
        self.set_carry_flag(result);
    }

    fn asl(&mut self, address: u16) {
        // Affects Flags: N Z C
        let parameter = self.as_bus().read_byte(address);
        let result = (parameter as u16) << 1;
        self.as_bus().write_byte(address, result as u8);

        self.set_negative_flag(result as u8);
        self.set_zero_flag(result as u8);
        self.set_carry_flag(result);
    }

    fn bit(&mut self, parameter: u8) {
        // Affects Flags: N V Z
        let result = self.cpu_state.reg_a & parameter;

        self.set_negative_flag(parameter); // neg if bit 7 in param is 1
        self.cpu_state
            .status
            .set(CpuStatus::OVERFLOW, parameter & 0b0100_0000 != 0); // overflow if bit 6 in param is 1
        self.set_zero_flag(result);
    }

    // Branching functions
    fn bpl(&mut self, parameter: u8) {
        self.cpu_state.branch_flag = !self.cpu_state.status.contains(CpuStatus::NEGATIVE);
        if self.cpu_state.branch_flag {
            // we need to left pad parameter with the bit 7 value
            // ex: 11111000 -> 1111111111111000
            let parameter = (parameter as i8) as u16;
            let new_program_counter = self.cpu_state.program_counter.wrapping_add(parameter);
            self.cpu_state.page_cross_flag =
                (new_program_counter >> 8) != (self.cpu_state.program_counter >> 8);
            self.cpu_state.program_counter = self.cpu_state.program_counter.wrapping_add(parameter);
        }
    }

    fn bmi(&mut self, parameter: u8) {
        self.cpu_state.branch_flag = self.cpu_state.status.contains(CpuStatus::NEGATIVE);
        if self.cpu_state.branch_flag {
            // we need to left pad parameter with the bit 7 value
            // ex: 11111000 -> 1111111111111000
            let parameter = (parameter as i8) as u16;
            let new_program_counter = self.cpu_state.program_counter.wrapping_add(parameter);
            self.cpu_state.page_cross_flag =
                (new_program_counter >> 8) != (self.cpu_state.program_counter >> 8);
            self.cpu_state.program_counter = self.cpu_state.program_counter.wrapping_add(parameter);
        }
    }

    fn bvc(&mut self, parameter: u8) {
        self.cpu_state.branch_flag = !self.cpu_state.status.contains(CpuStatus::OVERFLOW);
        if self.cpu_state.branch_flag {
            // we need to left pad parameter with the bit 7 value
            // ex: 11111000 -> 1111111111111000
            let parameter = (parameter as i8) as u16;
            let new_program_counter = self.cpu_state.program_counter.wrapping_add(parameter);
            self.cpu_state.page_cross_flag =
                (new_program_counter >> 8) != (self.cpu_state.program_counter >> 8);
            self.cpu_state.program_counter = self.cpu_state.program_counter.wrapping_add(parameter);
        }
    }

    fn bvs(&mut self, parameter: u8) {
        self.cpu_state.branch_flag = self.cpu_state.status.contains(CpuStatus::OVERFLOW);
        if self.cpu_state.branch_flag {
            // we need to left pad parameter with the bit 7 value
            // ex: 11111000 -> 1111111111111000
            let parameter = (parameter as i8) as u16;
            let new_program_counter = self.cpu_state.program_counter.wrapping_add(parameter);
            self.cpu_state.page_cross_flag =
                (new_program_counter >> 8) != (self.cpu_state.program_counter >> 8);
            self.cpu_state.program_counter = self.cpu_state.program_counter.wrapping_add(parameter);
        }
    }

    fn bcc(&mut self, parameter: u8) {
        self.cpu_state.branch_flag = !self.cpu_state.status.contains(CpuStatus::CARRY);
        if self.cpu_state.branch_flag {
            // we need to left pad parameter with the bit 7 value
            // ex: 11111000 -> 1111111111111000
            let parameter = (parameter as i8) as u16;
            let new_program_counter = self.cpu_state.program_counter.wrapping_add(parameter);
            self.cpu_state.page_cross_flag =
                (new_program_counter >> 8) != (self.cpu_state.program_counter >> 8);
            self.cpu_state.program_counter = self.cpu_state.program_counter.wrapping_add(parameter);
        }
    }

    fn bcs(&mut self, parameter: u8) {
        self.cpu_state.branch_flag = self.cpu_state.status.contains(CpuStatus::CARRY);
        if self.cpu_state.branch_flag {
            // we need to left pad parameter with the bit 7 value
            // ex: 11111000 -> 1111111111111000
            let parameter = (parameter as i8) as u16;
            let new_program_counter = self.cpu_state.program_counter.wrapping_add(parameter);
            self.cpu_state.page_cross_flag =
                (new_program_counter >> 8) != (self.cpu_state.program_counter >> 8);
            self.cpu_state.program_counter = self.cpu_state.program_counter.wrapping_add(parameter);
        }
    }

    fn bne(&mut self, parameter: u8) {
        self.cpu_state.branch_flag = !self.cpu_state.status.contains(CpuStatus::ZERO);
        if self.cpu_state.branch_flag {
            // we need to left pad parameter with the bit 7 value
            // ex: 11111000 -> 1111111111111000
            let parameter = (parameter as i8) as u16;
            let new_program_counter = self.cpu_state.program_counter.wrapping_add(parameter);
            self.cpu_state.page_cross_flag =
                (new_program_counter >> 8) != (self.cpu_state.program_counter >> 8);
            self.cpu_state.program_counter = self.cpu_state.program_counter.wrapping_add(parameter);
        }
    }

    fn beq(&mut self, parameter: u8) {
        self.cpu_state.branch_flag = self.cpu_state.status.contains(CpuStatus::ZERO);
        if self.cpu_state.branch_flag {
            // we need to left pad parameter with the bit 7 value
            // ex: 11111000 -> 1111111111111000
            let parameter = (parameter as i8) as u16;
            let new_program_counter = self.cpu_state.program_counter.wrapping_add(parameter);
            self.cpu_state.page_cross_flag =
                (new_program_counter >> 8) != (self.cpu_state.program_counter >> 8);
            self.cpu_state.program_counter = self.cpu_state.program_counter.wrapping_add(parameter);
        }
    }

    fn brk(&mut self) {
        // BRK causes a non-maskable interrupt and increments the program counter by one TODO figure out what this means
        // Affects Flags: B
        self.cpu_state.status.insert(CpuStatus::BRK);
    }

    fn cmp(&mut self, parameter: u8) {
        // Affects Flags: N Z C
        let result = self.cpu_state.reg_a.wrapping_sub(parameter);

        self.set_negative_flag(result);
        self.set_zero_flag(result);
        // Special carry flag case
        if self.cpu_state.reg_a >= parameter {
            self.cpu_state.status.insert(CpuStatus::CARRY);
        } else {
            self.cpu_state.status.remove(CpuStatus::CARRY);
        }
    }

    fn cpx(&mut self, parameter: u8) {
        // Affects Flags: N Z C
        let result = self.cpu_state.reg_x.wrapping_sub(parameter);

        self.set_negative_flag(result);
        self.set_zero_flag(result);
        // Special carry flag case
        if self.cpu_state.reg_x >= parameter {
            self.cpu_state.status.insert(CpuStatus::CARRY);
        } else {
            self.cpu_state.status.remove(CpuStatus::CARRY);
        }
    }

    fn cpy(&mut self, parameter: u8) {
        // Affects Flags: N Z C
        let result = self.cpu_state.reg_y.wrapping_sub(parameter);

        self.set_negative_flag(result);
        self.set_zero_flag(result);
        // Special carry flag case
        if self.cpu_state.reg_y >= parameter {
            self.cpu_state.status.insert(CpuStatus::CARRY);
        } else {
            self.cpu_state.status.remove(CpuStatus::CARRY);
        }
    }

    fn dec(&mut self, address: u16) {
        // Affects Flags: N Z
        let result = self.as_bus().read_byte(address).wrapping_sub(1);
        self.as_bus().write_byte(address, result);

        self.set_negative_flag(result);
        self.set_zero_flag(result);
    }

    fn eor(&mut self, parameter: u8) {
        // Affects Flags: N Z
        self.cpu_state.reg_a ^= parameter;

        self.set_negative_flag(self.cpu_state.reg_a);
        self.set_zero_flag(self.cpu_state.reg_a);
    }

    // flag instructions
    fn clc(&mut self) {
        // Clears carry flag
        self.cpu_state.status.remove(CpuStatus::CARRY);
    }

    fn sec(&mut self) {
        // Sets carry flag
        self.cpu_state.status.insert(CpuStatus::CARRY);
    }

    fn cli(&mut self) {
        // Clears interrupt flag
        self.cpu_state.status.remove(CpuStatus::INT_DISABLE);
    }

    fn sei(&mut self) {
        // Sets interrupt flag
        self.cpu_state.status.insert(CpuStatus::INT_DISABLE);
    }

    fn clv(&mut self) {
        // Clears overflow flag
        self.cpu_state.status.remove(CpuStatus::OVERFLOW);
    }

    fn cld(&mut self) {
        // Clears decimal flag
        self.cpu_state.status.remove(CpuStatus::DECIMAL);
    }

    fn sed(&mut self) {
        // Sets decimal flag
        self.cpu_state.status.insert(CpuStatus::DECIMAL);
    }

    fn inc(&mut self, address: u16) {
        // Affects Flags: N Z
        let result = self.as_bus().read_byte(address).wrapping_add(1);
        self.as_bus().write_byte(address, result);

        self.set_negative_flag(result);
        self.set_zero_flag(result);
    }

    fn jmp(&mut self, address: u16) {
        // Affects Flags: None
        self.cpu_state.program_counter = address;
    }

    fn jsr(&mut self, address: u16) {
        // Affects Flags: None
        let program_counter = self.cpu_state.program_counter - 1;
        let lsb = program_counter as u8;
        let msb = (program_counter >> 8) as u8;
        // Push msb first
        self.push_to_stack(msb);
        self.push_to_stack(lsb);

        self.cpu_state.program_counter = address;
    }

    fn lda(&mut self, parameter: u8) {
        // Affects Flags: N Z
        self.cpu_state.reg_a = parameter;

        self.set_negative_flag(self.cpu_state.reg_a);
        self.set_zero_flag(self.cpu_state.reg_a);
    }

    fn ldx(&mut self, parameter: u8) {
        // Affects Flags: N Z
        self.cpu_state.reg_x = parameter;

        self.set_negative_flag(self.cpu_state.reg_x);
        self.set_zero_flag(self.cpu_state.reg_x);
    }

    fn ldy(&mut self, parameter: u8) {
        // Affects Flags: N Z
        self.cpu_state.reg_y = parameter;

        self.set_negative_flag(self.cpu_state.reg_y);
        self.set_zero_flag(self.cpu_state.reg_y);
    }

    fn lsr_acc(&mut self, parameter: u8) {
        // Affects Flags: N Z C
        // LSR for accumulator
        self.cpu_state.reg_a = parameter >> 1;

        self.set_negative_flag(self.cpu_state.reg_a);
        self.set_zero_flag(self.cpu_state.reg_a);
        // Special carry flag case
        if parameter % 2 == 1 {
            self.cpu_state.status.insert(CpuStatus::CARRY);
        } else {
            self.cpu_state.status.remove(CpuStatus::CARRY);
        }
    }

    fn lsr(&mut self, address: u16) {
        // Affects Flags: N Z C
        // I think this writes to reg_a? Not sure
        let parameter = self.as_bus().read_byte(address);
        let result = parameter >> 1;
        self.as_bus().write_byte(address, result);

        self.set_negative_flag(result);
        self.set_zero_flag(result);
        // Special carry flag case
        if parameter % 2 == 1 {
            self.cpu_state.status.insert(CpuStatus::CARRY);
        } else {
            self.cpu_state.status.remove(CpuStatus::CARRY);
        }
    }

    fn ora(&mut self, parameter: u8) {
        // Affects Flags: N Z
        self.cpu_state.reg_a |= parameter;

        self.set_negative_flag(self.cpu_state.reg_a);
        self.set_zero_flag(self.cpu_state.reg_a);
    }

    fn tax(&mut self) {
        // Affects Flags: N Z
        self.cpu_state.reg_x = self.cpu_state.reg_a;

        self.set_negative_flag(self.cpu_state.reg_x);
        self.set_zero_flag(self.cpu_state.reg_x);
    }

    fn txa(&mut self) {
        // Affects Flags: N Z
        self.cpu_state.reg_a = self.cpu_state.reg_x;

        self.set_negative_flag(self.cpu_state.reg_a);
        self.set_zero_flag(self.cpu_state.reg_a);
    }

    fn dex(&mut self) {
        // Affects Flags: N Z
        self.cpu_state.reg_x = self.cpu_state.reg_x.wrapping_sub(1);

        self.set_negative_flag(self.cpu_state.reg_x);
        self.set_zero_flag(self.cpu_state.reg_x);
    }

    fn inx(&mut self) {
        // Affects Flags: N Z
        self.cpu_state.reg_x = self.cpu_state.reg_x.wrapping_add(1);

        self.set_negative_flag(self.cpu_state.reg_x);
        self.set_zero_flag(self.cpu_state.reg_x);
    }

    fn tay(&mut self) {
        // Affects Flags: N Z
        self.cpu_state.reg_y = self.cpu_state.reg_a;

        self.set_negative_flag(self.cpu_state.reg_y);
        self.set_zero_flag(self.cpu_state.reg_y);
    }

    fn tya(&mut self) {
        // Affects Flags: N Z
        self.cpu_state.reg_a = self.cpu_state.reg_y;

        self.set_negative_flag(self.cpu_state.reg_a);
        self.set_zero_flag(self.cpu_state.reg_a);
    }

    fn dey(&mut self) {
        // Affects Flags: N Z
        self.cpu_state.reg_y = self.cpu_state.reg_y.wrapping_sub(1);

        self.set_negative_flag(self.cpu_state.reg_y);
        self.set_zero_flag(self.cpu_state.reg_y);
    }

    fn iny(&mut self) {
        // Affects Flags: N Z
        self.cpu_state.reg_y = self.cpu_state.reg_y.wrapping_add(1);

        self.set_negative_flag(self.cpu_state.reg_y);
        self.set_zero_flag(self.cpu_state.reg_y);
    }

    fn rol_acc(&mut self, parameter: u8) {
        // Affects Flags: N Z C
        let mut result = (parameter as u16) << 1;
        if self.cpu_state.status.contains(CpuStatus::CARRY) {
            result += 1; // this should be safe from overflow
        }
        self.cpu_state.reg_a = result as u8;

        self.set_negative_flag(self.cpu_state.reg_a);
        self.set_zero_flag(self.cpu_state.reg_a);
        self.set_carry_flag(result);
    }

    fn rol(&mut self, address: u16) {
        // Affects Flags: N Z C
        let parameter = self.as_bus().read_byte(address);
        let mut result = (parameter as u16) << 1;
        if self.cpu_state.status.contains(CpuStatus::CARRY) {
            result += 1; // this should be safe from overflow
        }
        self.as_bus().write_byte(address, result as u8);

        self.set_negative_flag(result as u8);
        self.set_zero_flag(result as u8);
        self.set_carry_flag(result);
    }

    fn ror_acc(&mut self, parameter: u8) {
        // Affects Flags: N Z C
        let mut result = parameter >> 1;
        if self.cpu_state.status.contains(CpuStatus::CARRY) {
            result += 0b1000_0000;
        }
        self.cpu_state.reg_a = result;

        self.set_negative_flag(result);
        self.set_zero_flag(result);
        // Special carry flag case
        if parameter % 2 == 1 {
            self.cpu_state.status.insert(CpuStatus::CARRY);
        } else {
            self.cpu_state.status.remove(CpuStatus::CARRY);
        }
    }

    fn ror(&mut self, address: u16) {
        // Affects Flags: N Z C
        let parameter = self.as_bus().read_byte(address);
        let mut result = parameter >> 1;
        if self.cpu_state.status.contains(CpuStatus::CARRY) {
            result += 0b1000_0000;
        }
        self.as_bus().write_byte(address, result);

        self.set_negative_flag(result);
        self.set_zero_flag(result);
        // Special carry flag case
        if parameter % 2 == 1 {
            self.cpu_state.status.insert(CpuStatus::CARRY);
        } else {
            self.cpu_state.status.remove(CpuStatus::CARRY);
        }
    }

    fn rti(&mut self) {
        // Affected Flags: All
        self.plp(); // pop status from stack
        let lsb = self.pop_from_stack() as u16;
        let msb = self.pop_from_stack() as u16;
        self.cpu_state.program_counter = (msb << 8) + lsb;
    }

    fn rts(&mut self) {
        // Affected Flags: None
        let lsb = self.pop_from_stack() as u16;
        let msb = self.pop_from_stack() as u16;
        self.cpu_state.program_counter = (msb << 8) + lsb + 1;
    }

    fn sbc(&mut self, parameter: u8) {
        // Affects Flags: N V Z C
        // Can just use ADC internally
        self.adc(parameter ^ 0b1111_1111) // toggle every bit and pass to adc
    }

    fn txs(&mut self) {
        // Affects Flags: None
        // stack is in the reange 0x100 - 0x1FF
        self.cpu_state.stack_pointer = self.cpu_state.reg_x;
    }

    fn tsx(&mut self) {
        // Affects Flags: N Z
        self.cpu_state.reg_x = self.cpu_state.stack_pointer;

        self.set_negative_flag(self.cpu_state.reg_x);
        self.set_zero_flag(self.cpu_state.reg_x);
    }

    fn pha(&mut self) {
        // Affects Flags: None
        self.push_to_stack(self.cpu_state.reg_a);
    }

    fn pla(&mut self) {
        // Affects Flags: N Z
        self.cpu_state.reg_a = self.pop_from_stack();

        self.set_negative_flag(self.cpu_state.reg_a);
        self.set_zero_flag(self.cpu_state.reg_a);
    }

    fn php(&mut self) {
        // Affects Flags: None
        // Need to push 'status' with BRK set
        // https://www.nesdev.org/wiki/Status_flags#The_B_flag
        let status = self.cpu_state.status | CpuStatus::BRK;
        self.push_to_stack(status.bits());
    }

    fn plp(&mut self) {
        // Affects Flags: All
        self.cpu_state.status = CpuStatus::from_bits(self.pop_from_stack()).unwrap();
        // plp discards BRK flag
        // https://www.nesdev.org/wiki/Status_flags#The_B_flag
        self.cpu_state.status.remove(CpuStatus::BRK);
        self.cpu_state.status.insert(CpuStatus::ALWAYS);
    }

    fn sta(&mut self, address: u16) {
        // Affected Flags: None
        let value = self.cpu_state.reg_a;
        self.as_bus().write_byte(address, value);
    }

    fn stx(&mut self, address: u16) {
        // Affected Flags: None
        let value = self.cpu_state.reg_x;
        self.as_bus().write_byte(address, value);
    }

    fn sty(&mut self, address: u16) {
        // Affected Flags: None
        let value = self.cpu_state.reg_y;
        self.as_bus().write_byte(address, value);
    }
}
