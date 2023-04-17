#![allow(dead_code)]


use core::panic;
use std::{ops::Add, slice::Iter, fmt::{Display, self}};
use bitflags::bitflags;

use simple_logging::log_to_file;

use crate::traits::Memory;

pub mod bus;
use bus::Bus;

pub mod decode;
use decode::{Instruction, AddressingMode};

// #[derive(Debug)]
pub enum Param {    // used by an instruction
    Value(u8),
    Address(u16),
}

impl fmt::Debug for Param {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Param::Value(val) => write!(f, "Value(0x{:x})", val),
            Param::Address(addr) =>   write!(f, "Address(0x{:x})", addr),
        }
    }
}


// NOTE: all cpu opcodes are a single u8 of the form AAABBBCC in binary, BBB defines the addressing mode

bitflags! {
    #[derive(Debug, Clone)]
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

const STACK_POINTER_INIT: u8 = 0xfd;
const PROGRAM_COUNTER_INIT: u16 = 0x600;

#[derive(Debug)]
pub struct CPU {
    // General purpose registers
    pub reg_a: u8,
    pub reg_x: u8,
    pub reg_y: u8,
    // Special purpose registers
    pub status: CpuStatus,
    pub stack_pointer: u8,
    pub program_counter: u16,

    // Bus
    bus: Bus,
}

impl CPU {  // Decoding
    pub fn read_arg(&mut self, mode: &AddressingMode) -> Option<Param> {
        // Based on the addressing mode, read `n` number of argument bytes from the program and process it into a parameter
        // to be used by some instruction
        match mode {
            AddressingMode::Implicit => None,
            AddressingMode::Accumulator => {
                Some(Param::Value(self.reg_a))
            },
            AddressingMode::Immediate | AddressingMode::Relative => {
                Some(Param::Value(self.read_byte_from_pc()))
            },
            AddressingMode::IndirectJump => {
                // 6502 has a edge case with page boundary when performing indirect jumps
                // AN INDIRECT JUMP MUST NEVER USE A VECTOR BEGINNING ON THE LAST BYTE OF A PAGE
                // http://www.6502.org/tutorials/6502opcodes.html#JMP

                // if address $3000 contains $40, $30FF contains $80, and $3100 contains $50, 
                // the result of JMP ($30FF) will be a transfer of control to $4080 rather than $5080 as you intended 
                // i.e. the 6502 took the low byte of the address from $30FF and the high byte from $3000.

                // first read two bytes
                let mem_addr = self.read_two_bytes_from_pc();

                // read the two bytes from memory and form it into a mem addr
                let mem_addr = if mem_addr & 0x0FF == 0x0FF {
                    let lsb = self.read_byte(mem_addr) as u16;
                    let msb = self.read_byte(mem_addr & 0xFF00) as u16;
                    (msb << 8) + lsb
                } else {
                    self.read_two_bytes(mem_addr)
                };
                // now read from memory
                Some(Param::Address(mem_addr))
            },
            AddressingMode::Absolute => {
                // first read two bytes
                let mem_addr = self.read_two_bytes_from_pc();
                // read memory from bus
                Some(Param::Address(mem_addr))
            },
            AddressingMode::AbsoluteJump => {
                let mem_addr = self.read_two_bytes_from_pc();
                Some(Param::Address(mem_addr))
            },
            AddressingMode::ZeroPage => {
                // read single byte, msb is always 0x00
                let zero_page_addr = self.read_byte_from_pc() as u16;
                // read memory from bus
                Some(Param::Address(zero_page_addr))
            },
            AddressingMode::ZeroPageIndexX => {
                let zero_page_addr = self.read_byte_from_pc().wrapping_add(self.reg_x) as u16;
                Some(Param::Address(zero_page_addr))
            },
            AddressingMode::ZeroPageIndexY => {
                let zero_page_addr = self.read_byte_from_pc().wrapping_add(self.reg_y) as u16;
                Some(Param::Address(zero_page_addr))
            },
            AddressingMode::AbsoluteIndexX => {
                // Form <instruction> <addr>, X where <addr> is u16, specifies the value of read(<addr> + 1)
                let mem_addr = self.read_two_bytes_from_pc().wrapping_add(self.reg_x as u16);
                Some(Param::Address(mem_addr))
            },
            AddressingMode::AbsoluteIndexY => {
                // Same as AbsoluteIndexX, but with reg_y instead
                let mem_addr = self.read_two_bytes_from_pc().wrapping_add(self.reg_y as u16);
                Some(Param::Address(mem_addr))
            },
            AddressingMode::IndirectX => {
                // Form <instruction (<addr>, X), where <addr> is u8
                let zero_page_addr = (self.read_byte_from_pc().wrapping_add(self.reg_x)) as u16;
                // TODO: may need to re-evaluate how this is done when there's a page cross
                let indirect = self.read_two_page_bytes(zero_page_addr);
                // read memory from bus
                Some(Param::Address(indirect))
            },
            AddressingMode::IndirectY => {
                let zero_page_addr = self.read_byte_from_pc() as u16;
                // TODO: may need to re-evaluate how this is done when there's a page cross
                let indirect = self.read_two_page_bytes(zero_page_addr)
                                        .wrapping_add(self.reg_y as u16);
                Some(Param::Address(indirect))
            },
        }
    }
}

impl CPU {  // Public functions
    pub fn new_empty() -> Self {
        CPU::new(Bus::new_empty())
    }

    pub fn new(bus: Bus) -> Self {
        CPU {
            reg_a: 0,
            reg_x: 0,
            reg_y: 0,
            // status: CpuStatus::ALWAYS | CpuStatus::BRK,
            status: CpuStatus::ALWAYS | CpuStatus::INT_DISABLE,
            stack_pointer: STACK_POINTER_INIT,      // probably needs to initialize to something else
            program_counter: PROGRAM_COUNTER_INIT,      // same here
            bus: bus,
        }
    }

    pub fn reset(&mut self) {
        self.reg_a = 0;
        self.reg_x = 0;
        self.reg_y = 0;
        self.stack_pointer = STACK_POINTER_INIT;
        // self.status = CpuStatus::ALWAYS | CpuStatus::BRK;
        self.status = CpuStatus::ALWAYS | CpuStatus::INT_DISABLE;
        self.program_counter = self.read_two_bytes(0xFFFC) - 4; // TEST: trying out subtracting one
    }

    pub fn load_nes(&mut self, path: &str) {
        self.bus.load_nes(path);
    }

    pub fn run_nes(&mut self, path: &str) -> Result<(), String>  {
        self.run_nes_with_callback(path, |_| {})
    }

    pub fn run_nes_with_callback<F>(&mut self, path: &str, mut callback: F) -> Result<(), String> 
    where
        F: FnMut(&mut CPU)
    {
        self.load_nes(path);
        self.reset();
        println!("Program Counter after reset: {:x}", self.program_counter);
        self.run_with_callback(callback)
    }

    pub fn load_program(&mut self, program: Vec<u8>) {
        // Write program to RAM, starting at 0x0600
        for i in 0..(program.len() as u16) {
            self.write_byte(0x0600 + i, program[i as usize]);
        }
    }

    pub fn run_program(&mut self, program: Vec<u8>) -> Result<(), String> {
        self.run_program_with_callback(program, |_| {})
    }

    pub fn run_program_with_callback<F>(&mut self, program: Vec<u8>, mut callback: F) -> Result<(), String>
    where
        F: FnMut(&mut CPU)
    {
        self.load_program(program);
        self.reset();
        self.program_counter = 0x0600;
        self.run_with_callback(callback)
    }

    pub fn run(&mut self) -> Result<(), String> {
        self.run_with_callback(|_| {})
    }

    pub fn run_with_callback<F>(&mut self, mut callback: F) -> Result<(), String> 
    where
        F: FnMut(&mut CPU)
    { 
        // This function will take in a program, and execute it step by step
        // TODO: result is Result<()), String> right now, need to change to something more descriptive
        loop {
            // 0. Execute callback
            callback(self);

            // 1. Read opcode and decode it to an instruction, always takes 1 cycle
            let opcode_raw = self.read_byte_from_pc();
            let (instruction, addressing_mode) = decode::decode_opcode(opcode_raw)?;

            // TEMPORARY: if BRK, then exit
            if instruction == Instruction::BRK {
                return Ok(())
            }

            // 2. Read some number of bytes depending on what the addressing mode is and decode the instruction parameter, may take many cycles
            // Ref: http://www.6502.org/tutorials/6502opcodes.html
            let parameter = self.read_arg(&addressing_mode);
            
            // 3. Execute the instruction
            self.execute_instruction(instruction, parameter);
        }
    }

    pub fn execute_instruction(&mut self, instruction: Instruction, parameter: Option<Param>) {
        // FUTURE WORK: can probably condense this more, but not really necessary
        match instruction {
            Instruction::ADC => {
                match parameter {
                    Some(Param::Value(val)) => 
                        self.adc(val),
                    Some(Param::Address(mem_addr)) => 
                        self.adc(self.read_byte(mem_addr)),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::AND => {
                match parameter {
                    Some(Param::Value(val)) => 
                        self.and(val),
                    Some(Param::Address(mem_addr)) => 
                        self.and(self.read_byte(mem_addr)),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::ASL => {
                match parameter {
                    Some(Param::Value(val)) => 
                        self.asl_acc(val),
                    Some(Param::Address(mem_addr)) => 
                        self.asl(mem_addr),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::BIT => {
                match parameter {
                    Some(Param::Value(val)) => 
                        self.bit(val),
                    Some(Param::Address(mem_addr)) => 
                        self.bit(self.read_byte(mem_addr)),
                    _ => panic!("Invalid parameter"),
                }
            }
            // Add branching here
            Instruction::BPL => {
                match parameter {
                    Some(Param::Value(val)) => 
                        self.bpl(val),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::BMI => {
                match parameter {
                    Some(Param::Value(val)) => 
                        self.bmi(val),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::BVC => {
                match parameter {
                    Some(Param::Value(val)) => 
                        self.bvc(val),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::BVS => {
                match parameter {
                    Some(Param::Value(val)) => 
                        self.bvs(val),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::BCC => {
                match parameter {
                    Some(Param::Value(val)) => 
                        self.bcc(val),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::BCS => {
                match parameter {
                    Some(Param::Value(val)) => 
                        self.bcs(val),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::BNE => {
                match parameter {
                    Some(Param::Value(val)) => 
                        self.bne(val),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::BEQ => {
                match parameter {
                    Some(Param::Value(val)) => 
                        self.beq(val),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::BRK => {
                match parameter {
                    None =>
                        self.brk(),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::CMP => {
                match parameter {
                    Some(Param::Value(val)) => 
                        self.cmp(val),
                    Some(Param::Address(mem_addr)) => 
                        self.cmp(self.read_byte(mem_addr)),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::CPX => {
                match parameter {
                    Some(Param::Value(val)) => 
                        self.cpx(val),
                    Some(Param::Address(mem_addr)) => 
                        self.cpx(self.read_byte(mem_addr)),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::CPY => {
                match parameter {
                    Some(Param::Value(val)) => 
                        self.cpy(val),
                    Some(Param::Address(mem_addr)) => 
                        self.cpy(self.read_byte(mem_addr)),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::DEC => {
                match parameter {
                    Some(Param::Address(mem_addr)) => 
                        self.dec(mem_addr),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::EOR => {
                match parameter {
                    Some(Param::Value(val)) => 
                        self.eor(val),
                    Some(Param::Address(mem_addr)) => 
                        self.eor(self.read_byte(mem_addr)),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::CLC => {
                match parameter {
                    None => 
                        self.clc(),
                    _ => panic!("Invalid parameter")
                }
            },
            Instruction::SEC => {
                match parameter {
                    None => 
                        self.sec(),
                    _ => panic!("Invalid parameter")
                }
            },
            Instruction::CLI => {
                match parameter {
                    None => 
                        self.cli(),
                    _ => panic!("Invalid parameter")
                }
            },
            Instruction::SEI => {
                match parameter {
                    None => 
                        self.sei(),
                    _ => panic!("Invalid parameter")
                }
            },
            Instruction::CLV => {
                match parameter {
                    None => 
                        self.clv(),
                    _ => panic!("Invalid parameter")
                }
            },
            Instruction::CLD => {
                match parameter {
                    None => 
                        self.cld(),
                    _ => panic!("Invalid parameter")
                }
            },
            Instruction::SED => {
                match parameter {
                    None => 
                        self.sed(),
                    _ => panic!("Invalid parameter")
                }
            },
            Instruction::INC => {
                match parameter {
                    Some(Param::Address(mem_addr)) => 
                        self.inc(mem_addr),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::JMP => {
                match parameter {
                    Some(Param::Address(mem_addr)) => 
                        self.jmp(mem_addr),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::JSR => {
                match parameter {
                    Some(Param::Address(mem_addr)) => 
                        self.jsr(mem_addr),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::LDA => {
                match parameter {
                    Some(Param::Value(val)) => 
                        self.lda(val),
                    Some(Param::Address(mem_addr)) => 
                        self.lda(self.read_byte(mem_addr)),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::LDX => {
                match parameter {
                    Some(Param::Value(val)) => 
                        self.ldx(val),
                    Some(Param::Address(mem_addr)) => 
                        self.ldx(self.read_byte(mem_addr)),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::LDY => {
                match parameter {
                    Some(Param::Value(val)) => 
                        self.ldy(val),
                    Some(Param::Address(mem_addr)) => 
                        self.ldy(self.read_byte(mem_addr)),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::LSR => {
                match parameter {
                    // This should only ever be used for accumulator addressing mode
                    Some(Param::Value(val)) => 
                        self.lsr_acc(val),
                    Some(Param::Address(mem_addr)) => 
                        self.lsr(mem_addr),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::NOP => {

            }
            Instruction::ORA => {
                match parameter {
                    Some(Param::Value(val)) => 
                        self.ora(val),
                    Some(Param::Address(mem_addr)) => 
                        self.ora(self.read_byte(mem_addr)),
                    _ => panic!("Invalid parameter"),
                }
            },
            // Register instructions
            Instruction::TAX => {
                match parameter {
                    None => 
                        self.tax(),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::TXA => {
                match parameter {
                    None => 
                        self.txa(),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::DEX => {
                match parameter {
                    None => 
                        self.dex(),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::INX => {
                match parameter {
                    None => 
                        self.inx(),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::TAY => {
                match parameter {
                    None => 
                        self.tay(),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::TYA => {
                match parameter {
                    None => 
                        self.tya(),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::DEY => {
                match parameter {
                    None => 
                        self.dey(),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::INY => {
                match parameter {
                    None => 
                        self.iny(),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::ROL => {
                match parameter {
                    Some(Param::Value(val)) => 
                        self.rol_acc(val),
                    Some(Param::Address(mem_addr)) => 
                        self.rol(mem_addr),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::ROR => {
                match parameter {
                    Some(Param::Value(val)) => 
                        self.ror_acc(val),
                    Some(Param::Address(mem_addr)) => 
                        self.ror(mem_addr),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::RTI => {
                match parameter {
                    None => 
                        self.rti(),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::RTS => {
                match parameter {
                    None => 
                        self.rts(),
                    _ => panic!("Invalid parameter"),
                }
            }
            Instruction::SBC => {
                match parameter {
                    Some(Param::Value(val)) => 
                        self.sbc(val),
                    Some(Param::Address(mem_addr)) => 
                        self.sbc(self.read_byte(mem_addr)),
                    _ => panic!("Invalid parameter"),
                }
            },
            // Stack instructions
            Instruction::TXS => {
                match parameter {
                    None => 
                        self.txs(),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::TSX => {
                match parameter {
                    None => 
                        self.tsx(),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::PHA => {
                match parameter {
                    None => 
                        self.pha(),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::PLA => {
                match parameter {
                    None => 
                        self.pla(),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::PHP => {
                match parameter {
                    None => 
                        self.php(),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::PLP => {
                match parameter {
                    None => 
                        self.plp(),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::STA => {
                match parameter {
                    Some(Param::Address(mem_addr)) => 
                        self.sta(mem_addr),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::STX => {
                match parameter {
                    Some(Param::Address(mem_addr)) => 
                        self.stx(mem_addr),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::STY => {
                match parameter {
                    Some(Param::Address(mem_addr)) => 
                        self.sty(mem_addr),
                    _ => panic!("Invalid parameter"),
                }
            },
            _ => panic!("Not implemented"),
        }
    }
}

impl Memory for CPU {
    fn read_byte(&self, index: u16) -> u8 {
        // must increment program counter before the attempted read returns None
        self.bus.read_byte(index)
    }

    fn write_byte(&mut self, index: u16, value: u8) {
        self.bus.write_byte(index, value)
    }
}
impl CPU {  // helper functions

    pub fn get_status(&self) -> u8 {
        self.status.bits()
    }

    pub fn read_byte_from_pc(&mut self) -> u8 {
        let read_addr = self.program_counter;
        self.program_counter += 1;
        self.read_byte(read_addr)
    }

    fn read_two_bytes_from_pc(&mut self) -> u16 {
        let read_addr = self.program_counter;
        self.program_counter += 2;
        self.read_two_bytes(read_addr)
    }

    fn push_to_stack(&mut self, value: u8) {
        // Stack located from 0x100 to 0x1FF, growing downward
        // For push, need to write first, then decrement
        let stack_addr = 0x100 + (self.stack_pointer as u16);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
        self.write_byte(stack_addr, value)
    }

    fn pop_from_stack(&mut self) -> u8 {
        // For pop, need to increment first, then read
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        let stack_addr = 0x100 + (self.stack_pointer as u16);
        self.read_byte(stack_addr)
    }

    fn set_zero_flag(&mut self, result: u8) {
        if result == 0 {
            self.status.insert(CpuStatus::ZERO);
        } else {
            self.status.remove(CpuStatus::ZERO);
        }
    }

    fn set_negative_flag(&mut self, result: u8) {
        if result & 0b1000_0000 != 0 {
            self.status.insert(CpuStatus::NEGATIVE);
        } else {
            self.status.remove(CpuStatus::NEGATIVE);
        }
    }

    fn set_carry_flag(&mut self, result: u16) {
        // Check carry flag
        if result > 0xFF {
            self.status.insert(CpuStatus::CARRY);
        } else {
            self.status.remove(CpuStatus::CARRY);
        }
    } 

}

impl CPU {  // implement specific ISA instructions
    fn adc(&mut self, parameter: u8) {
        /// Affects Flags: N V Z C

        // Cast all relevant values to u16
        let reg_a = self.reg_a as u16;
        let val = parameter as u16;
        let carry = self.status.contains(CpuStatus::CARRY) as u16;

        // Add them together
        let sum = reg_a + val + carry;

        // Keep only least significant byte for result
        let result = sum as u8;

        self.set_negative_flag(result);

        // Check overflow flag; bit 7 must match for operands and result
        if (parameter ^ result) & (self.reg_a ^ result) & 0b1000_0000 != 0 {
            self.status.insert(CpuStatus::OVERFLOW);
        } else {
            self.status.remove(CpuStatus::OVERFLOW);
        }

        self.set_zero_flag(result);
        self.set_carry_flag(sum);
        
        // Set accumulator
        self.reg_a = result;
    }

    fn and(&mut self, parameter: u8) {
        // Affects Flags: N Z
        self.reg_a = self.reg_a & parameter;

        self.set_negative_flag(self.reg_a);
        self.set_zero_flag(self.reg_a);
    }

    fn asl_acc(&mut self, parameter: u8) {
        // Affects Flags: N Z C

        let result = (parameter as u16) << 1;
        self.reg_a = result as u8;

        self.set_negative_flag(self.reg_a);
        self.set_zero_flag(self.reg_a);
        self.set_carry_flag(result);
    }

    fn asl(&mut self, address: u16) {
        // Affects Flags: N Z C
        let parameter = self.read_byte(address);
        let result = (parameter as u16) << 1;
        self.write_byte(address, result as u8);

        self.set_negative_flag(result as u8);
        self.set_zero_flag(result as u8);
        self.set_carry_flag(result);
    }

    fn bit(&mut self, parameter: u8) {
        // Affects Flags: N V Z
        let result = self.reg_a & parameter;

        self.set_negative_flag(parameter); // neg if bit 7 in param is 1
        self.status.set(CpuStatus::OVERFLOW, parameter & 0b0100_0000 != 0); // overflow if bit 6 in param is 1
        self.set_zero_flag(result);
        
    }

    // Branching functions
    fn bpl(&mut self, parameter: u8) {
        if !self.status.contains(CpuStatus::NEGATIVE) {
            // we need to left pad parameter with the bit 7 value
            // ex: 11111000 -> 1111111111111000
            let parameter = (parameter as i8) as u16;
            self.program_counter = self.program_counter.wrapping_add(parameter);
        }
    }

    fn bmi(&mut self, parameter: u8) {
        if self.status.contains(CpuStatus::NEGATIVE) {
            // we need to left pad parameter with the bit 7 value
            // ex: 11111000 -> 1111111111111000
            let parameter = (parameter as i8) as u16;
            self.program_counter = self.program_counter.wrapping_add(parameter);
        }
    }

    fn bvc(&mut self, parameter: u8) {
        if !self.status.contains(CpuStatus::OVERFLOW) {
            // we need to left pad parameter with the bit 7 value
            // ex: 11111000 -> 1111111111111000
            let parameter = (parameter as i8) as u16;
            self.program_counter = self.program_counter.wrapping_add(parameter);
        }
    }

    fn bvs(&mut self, parameter: u8) {
        if self.status.contains(CpuStatus::OVERFLOW) {
            // we need to left pad parameter with the bit 7 value
            // ex: 11111000 -> 1111111111111000
            let parameter = (parameter as i8) as u16;
            self.program_counter = self.program_counter.wrapping_add(parameter);
        }
    }

    fn bcc(&mut self, parameter: u8) {
        if !self.status.contains(CpuStatus::CARRY) {
            // we need to left pad parameter with the bit 7 value
            // ex: 11111000 -> 1111111111111000
            let parameter = (parameter as i8) as u16;
            self.program_counter = self.program_counter.wrapping_add(parameter);
        }
    }

    fn bcs(&mut self, parameter: u8) {
        if self.status.contains(CpuStatus::CARRY) {
            // we need to left pad parameter with the bit 7 value
            // ex: 11111000 -> 1111111111111000
            let parameter = (parameter as i8) as u16;
            self.program_counter = self.program_counter.wrapping_add(parameter);
        }
    }

    fn bne(&mut self, parameter: u8) {
        if !self.status.contains(CpuStatus::ZERO) {
            // we need to left pad parameter with the bit 7 value
            // ex: 11111000 -> 1111111111111000
            let parameter = (parameter as i8) as u16;
            self.program_counter = self.program_counter.wrapping_add(parameter);
        }
    }

    fn beq(&mut self, parameter: u8) {
        if self.status.contains(CpuStatus::ZERO) {
            // we need to left pad parameter with the bit 7 value
            // ex: 11111000 -> 1111111111111000
            let parameter = (parameter as i8) as u16;
            self.program_counter = self.program_counter.wrapping_add(parameter);
        }
    }

    fn brk(&mut self) {
        // BRK causes a non-maskable interrupt and increments the program counter by one TODO figure out what this means
        // Affects Flags: B
        self.status.insert(CpuStatus::BRK);
    }

    fn cmp(&mut self, parameter: u8) {
        // Affects Flags: N Z C
        let result = self.reg_a.wrapping_sub(parameter);

        self.set_negative_flag(result);
        self.set_zero_flag(result);
        // Special carry flag case
        if self.reg_a >= parameter {
            self.status.insert(CpuStatus::CARRY);
        } else {
            self.status.remove(CpuStatus::CARRY);
        }
    }

    fn cpx(&mut self, parameter: u8) {
        // Affects Flags: N Z C
        let result = self.reg_x.wrapping_sub(parameter);

        self.set_negative_flag(result);
        self.set_zero_flag(result);
        // Special carry flag case
        if self.reg_x >= parameter {
            self.status.insert(CpuStatus::CARRY);
        } else {
            self.status.remove(CpuStatus::CARRY);
        }
    }

    fn cpy(&mut self, parameter: u8) {
        // Affects Flags: N Z C
        let result = self.reg_y.wrapping_sub(parameter);

        self.set_negative_flag(result);
        self.set_zero_flag(result);
        // Special carry flag case
        if self.reg_y >= parameter {
            self.status.insert(CpuStatus::CARRY);
        } else {
            self.status.remove(CpuStatus::CARRY);
        }
    }

    fn dec(&mut self, address: u16) {
        // Affects Flags: N Z
        let result = self.read_byte(address).wrapping_sub(1);
        self.write_byte(address, result);

        self.set_negative_flag(result);
        self.set_zero_flag(result);
    }

    fn eor(&mut self, parameter: u8) {
        // Affects Flags: N Z
        self.reg_a = self.reg_a ^ parameter;

        self.set_negative_flag(self.reg_a);
        self.set_zero_flag(self.reg_a);
    }

    // flag instructions
    fn clc(&mut self) {
        // Clears carry flag
        self.status.remove(CpuStatus::CARRY);
    }

    fn sec(&mut self) {
        // Sets carry flag
        self.status.insert(CpuStatus::CARRY);
    }

    fn cli(&mut self) {
        // Clears interrupt flag
        self.status.remove(CpuStatus::INT_DISABLE);
    }

    fn sei(&mut self) {
        // Sets interrupt flag
        self.status.insert(CpuStatus::INT_DISABLE);
    }

    fn clv(&mut self) {
        // Clears overflow flag
        self.status.remove(CpuStatus::OVERFLOW);
    }

    fn cld(&mut self) {
        // Clears decimal flag
        self.status.remove(CpuStatus::DECIMAL);
    }

    fn sed(&mut self) {
        // Sets decimal flag
        self.status.insert(CpuStatus::DECIMAL);
    }

    fn inc(&mut self, address: u16) {
        // Affects Flags: N Z
        let result = self.read_byte(address).wrapping_add(1);
        self.write_byte(address, result);

        self.set_negative_flag(result);
        self.set_zero_flag(result);
    }
    
    fn jmp(&mut self, address: u16) {
        // Affects Flags: None
        self.program_counter = address;
    }

    fn jsr(&mut self, address: u16) {
        // Affects Flags: None
        let program_counter = self.program_counter - 1;
        let lsb = program_counter as u8;
        let msb = (program_counter >> 8) as u8;
        // Push msb first
        self.push_to_stack(msb);
        self.push_to_stack(lsb);

        self.program_counter = address;
    }

    fn lda(&mut self, parameter: u8) {
        // Affects Flags: N Z
        self.reg_a = parameter;

        self.set_negative_flag(self.reg_a);
        self.set_zero_flag(self.reg_a);
    }

    fn ldx(&mut self, parameter: u8) {
        // Affects Flags: N Z
        self.reg_x = parameter;

        self.set_negative_flag(self.reg_x);
        self.set_zero_flag(self.reg_x);
    }

    fn ldy(&mut self, parameter: u8) {
        // Affects Flags: N Z
        self.reg_y = parameter;

        self.set_negative_flag(self.reg_y);
        self.set_zero_flag(self.reg_y);
    }

    fn lsr_acc(&mut self, parameter: u8) {
        // Affects Flags: N Z C
        // LSR for accumulator
        self.reg_a = parameter >> 1;

        self.set_negative_flag(self.reg_a);
        self.set_zero_flag(self.reg_a);
        // Special carry flag case
        if parameter % 2 == 1 {
            self.status.insert(CpuStatus::CARRY);
        } else {
            self.status.remove(CpuStatus::CARRY);
        }
    }

    fn lsr(&mut self, address: u16) {
        // Affects Flags: N Z C
        // I think this writes to reg_a? Not sure
        let parameter = self.read_byte(address);
        let result = parameter >> 1;
        self.write_byte(address, result);

        self.set_negative_flag(result);
        self.set_zero_flag(result);
        // Special carry flag case
        if parameter % 2 == 1 {
            self.status.insert(CpuStatus::CARRY);
        } else {
            self.status.remove(CpuStatus::CARRY);
        }
    }

    fn ora(&mut self, parameter: u8) {
        // Affects Flags: N Z
        self.reg_a = self.reg_a | parameter;

        self.set_negative_flag(self.reg_a);
        self.set_zero_flag(self.reg_a);
    }

    fn tax(&mut self) {
        // Affects Flags: N Z
        self.reg_x = self.reg_a;

        self.set_negative_flag(self.reg_x);
        self.set_zero_flag(self.reg_x);
    }

    fn txa(&mut self) {
        // Affects Flags: N Z
        self.reg_a = self.reg_x;

        self.set_negative_flag(self.reg_a);
        self.set_zero_flag(self.reg_a);
    }

    fn dex(&mut self) {
        // Affects Flags: N Z
        self.reg_x = self.reg_x.wrapping_sub(1);

        self.set_negative_flag(self.reg_x);
        self.set_zero_flag(self.reg_x);
    }

    fn inx(&mut self) {
        // Affects Flags: N Z
        self.reg_x = self.reg_x.wrapping_add(1);

        self.set_negative_flag(self.reg_x);
        self.set_zero_flag(self.reg_x);
    }

    fn tay(&mut self) {
        // Affects Flags: N Z
        self.reg_y = self.reg_a;

        self.set_negative_flag(self.reg_y);
        self.set_zero_flag(self.reg_y);
    }

    fn tya(&mut self) {
        // Affects Flags: N Z
        self.reg_a = self.reg_y;

        self.set_negative_flag(self.reg_a);
        self.set_zero_flag(self.reg_a);
    }

    fn dey(&mut self) {
        // Affects Flags: N Z
        self.reg_y = self.reg_y.wrapping_sub(1);

        self.set_negative_flag(self.reg_y);
        self.set_zero_flag(self.reg_y);
    }

    fn iny(&mut self) {
        // Affects Flags: N Z
        self.reg_y = self.reg_y.wrapping_add(1);

        self.set_negative_flag(self.reg_y);
        self.set_zero_flag(self.reg_y);
    }

    fn rol_acc(&mut self, parameter: u8) {
        // Affects Flags: N Z C
        let mut result = (parameter as u16) << 1;
        if self.status.contains(CpuStatus::CARRY) {
            result += 1;    // this should be safe from overflow
        }
        self.reg_a = result as u8;

        self.set_negative_flag(self.reg_a);
        self.set_zero_flag(self.reg_a);
        self.set_carry_flag(result);
    }

    fn rol(&mut self, address: u16) {
        // Affects Flags: N Z C
        let parameter = self.read_byte(address);
        let mut result = (parameter as u16) << 1;
        if self.status.contains(CpuStatus::CARRY) {
            result += 1;    // this should be safe from overflow
        }
        self.write_byte(address, result as u8);

        self.set_negative_flag(result as u8);
        self.set_zero_flag(result as u8);
        self.set_carry_flag(result);
    }

    fn ror_acc(&mut self, parameter: u8) {
        // Affects Flags: N Z C
        let mut result = parameter >> 1;
        if self.status.contains(CpuStatus::CARRY) {
            result += 0b1000_0000;
        }
        self.reg_a = result;
        
        self.set_negative_flag(result);
        self.set_zero_flag(result);
        // Special carry flag case
        if parameter % 2 == 1 {
            self.status.insert(CpuStatus::CARRY);
        } else {
            self.status.remove(CpuStatus::CARRY);
        }
    }

    fn ror(&mut self, address: u16) {
        // Affects Flags: N Z C
        let parameter = self.read_byte(address);
        let mut result = parameter >> 1;
        if self.status.contains(CpuStatus::CARRY) {
            result += 0b1000_0000;
        }
        self.write_byte(address, result);
        
        self.set_negative_flag(result);
        self.set_zero_flag(result);
        // Special carry flag case
        if parameter % 2 == 1 {
            self.status.insert(CpuStatus::CARRY);
        } else {
            self.status.remove(CpuStatus::CARRY);
        }
    }

    fn rti(&mut self) {
        // Affected Flags: All
        self.plp();     // pop status from stack
        let lsb = self.pop_from_stack() as u16;
        let msb = self.pop_from_stack() as u16;
        self.program_counter = (msb << 8) + lsb;
    }

    fn rts(&mut self) {
        // Affected Flags: None
        let lsb = self.pop_from_stack() as u16;
        let msb = self.pop_from_stack() as u16;
        self.program_counter = (msb << 8) + lsb + 1;
    }

    fn sbc(&mut self, parameter: u8) {
        // Affects Flags: N V Z C
        // Can just use ADC internally
        self.adc(parameter ^ 0b1111_1111) // toggle every bit and pass to adc
    }   

    fn txs(&mut self) {
        // Affects Flags: None
        // stack is in the reange 0x100 - 0x1FF
        self.stack_pointer = self.reg_x;
    }

    fn tsx(&mut self) {
        // Affects Flags: N Z
        self.reg_x = self.stack_pointer;

        self.set_negative_flag(self.reg_x);
        self.set_zero_flag(self.reg_x);
    }

    fn pha(&mut self) {
        // Affects Flags: None
        self.push_to_stack(self.reg_a);
    }

    fn pla(&mut self) {
        // Affects Flags: N Z
        self.reg_a = self.pop_from_stack();

        self.set_negative_flag(self.reg_a);
        self.set_zero_flag(self.reg_a);
    }

    fn php(&mut self) {
        // Affects Flags: None
        // Need to push 'status' with BRK set
        // https://www.nesdev.org/wiki/Status_flags#The_B_flag
        let status = self.status.clone() | CpuStatus::BRK;
        self.push_to_stack(status.bits());
    }

    fn plp(&mut self) {
        // Affects Flags: All
        self.status = CpuStatus::from_bits(self.pop_from_stack()).unwrap();
        // plp discards BRK flag
        // https://www.nesdev.org/wiki/Status_flags#The_B_flag
        self.status.remove(CpuStatus::BRK);
        self.status.insert(CpuStatus::ALWAYS);
    }

    fn sta(&mut self, address: u16) {
        // Affected Flags: None
        self.write_byte(address, self.reg_a);
    }

    fn stx(&mut self, address: u16) {
        // Affected Flags: None
        self.write_byte(address, self.reg_x);
    }

    fn sty(&mut self, address: u16) {
        // Affected Flags: None
        self.write_byte(address, self.reg_y);
    }
}
