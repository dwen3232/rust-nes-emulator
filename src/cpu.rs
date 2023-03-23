use core::panic;
use std::{ops::Add, slice::Iter};
use crate::bus::Bus;

use bitflags::bitflags;

pub fn some_function() -> i32 {
    // this is just an example function for testing
    5
}

pub enum Instruction {
    ADC,
    LDA,
    TAX,
    INX,
}
pub enum AddressingMode {
    Implicit,           // implicit
    Accumulator,        // val = A
    Immediate,          // val = arg8  
    IndirectJump,       // val = peek(arg16), only used by JMP
    Relative,           // val = arg8, offset
    Absolute,           // val = peek(arg16)
    AbsoluteJump,       // val = arg16, only used by JMP (I think, also this might be wrong)
    ZeroPage,           // val = peek(arg8)
    ZeroPageIndexX,     // val = peek((arg8 + X) % 256)
    ZeroPageIndexY,     // val = peek((arg8 + Y) % 256)
    AbsoluteIndexX,     // val = peek(arg16 + X)
    AbsoluteIndexY,     // val = peek(arg16 + Y)
    IndirectX,          // val = peek(peek((arg + X) % 256) + PEEK((arg + X + 1) % 256) * 256)
    IndirectY,          // val = peek(peek((arg + X) % 256) + PEEK((arg + X + 1) % 256) * 256)
}

// NOTE: all cpu opcodes are a single u8 of the form AAABBBCC in binary, BBB defines the addressing mode

bitflags! {
    struct CpuStatus: u8 {
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





pub struct CPU {
    // General purpose registers
    pub reg_a: u8,
    pub reg_x: u8,
    pub reg_y: u8,
    // Special purpose registers
    status: CpuStatus,
    stack_pointer: u16,
    program_counter: u8,

    // Bus
    bus: Bus,
}

const ADDRESSING_MODE_MASK: u8 = 0b0001_1100;
const INSTRUCTION_MASK: u8 = 0b1110_0011;

impl CPU {
    
    pub fn new() -> Self {
        CPU {
            reg_a: 0,
            reg_x: 0,
            reg_y: 0,
            status: CpuStatus::ALWAYS,
            stack_pointer: 0,      // probably needs to initialize to something else
            program_counter: 0,      // same here
            bus: Bus::new(),
        }
    }

    pub fn decode_opcode(&self, opcode: u8) -> (Instruction, AddressingMode) {
        /// Opcodes are u8 integers of the form AAABBBCC in binary, where BBB defines 
        /// the addressing mode and AAACC defines the instruction.
        /// Ref: https://en.wikibooks.org/wiki/6502_Assembly
        /// Ref: https://www.nesdev.org/wiki/CPU_unofficial_opcodes

        let addressing_mode_raw = opcode & ADDRESSING_MODE_MASK;  // this is 000BBB00
        let instruction_raw = opcode & INSTRUCTION_MASK; // this is 000
        let instruction = match instruction_raw {
            0b01100001 => Instruction::ADC,
            _ => panic!("Not implemented")
        };

        let addressing_mode = match (&instruction, addressing_mode_raw) {
            (Instruction::ADC, mode) => match mode {
                0b000 => AddressingMode::IndirectX,
                0b001 => AddressingMode::ZeroPage,
                0b010 => AddressingMode::Immediate,
                0b011 => AddressingMode::Absolute,
                0b100 => AddressingMode::IndirectY,
                0b101 => AddressingMode::ZeroPageIndexX,
                0b110 => AddressingMode::AbsoluteIndexY,
                0b111 => AddressingMode::AbsoluteIndexX,
                _ => panic!("Not implemented")
            },
            _ => panic!("Not implemented")
        };
        (instruction, addressing_mode)
    }


    fn read_arg(&mut self, mode: &AddressingMode, program: &Vec<u8>) -> Option<u8> {
        /// Based on the addressing mode, read `n` number of argument bytes from the program and process it into a parameter
        /// to be used by some instruction

        match mode {
            AddressingMode::Implicit => None,
            AddressingMode::Accumulator => Some(self.reg_a),
            AddressingMode::Immediate => {
                let byte = *program.get(self.program_counter as usize)?;
                self.program_counter += 1;
                Some(byte)
            },
            AddressingMode::IndirectJump => {
                // first read two bytes
                let lsb = *program.get(self.program_counter as usize)? as u16;
                let msb = *program.get((self.program_counter + 1) as usize)? as u16;
                self.program_counter += 2;
                // translate it into a u16 address
                let mem_addr = (msb << 8) + lsb;
                // read the two bytes from memory and form it into a mem addr
                let lsb = self.bus.read(mem_addr) as u16;
                let msb = self.bus.read(mem_addr + 1) as u16;
                let mem_addr = (msb << 8) + lsb;
                // now read from memory
                Some(self.bus.read(mem_addr))
            },
            AddressingMode::Relative => panic!("Not implemented"),
            AddressingMode::Absolute => {
                // first read two bytes
                let lsb = *program.get(self.program_counter as usize)? as u16;
                let msb = *program.get((self.program_counter + 1) as usize)? as u16;
                self.program_counter += 2;
                // translate it into a u16 address
                let mem_addr = (msb << 8) + lsb;
                // read it directly from memory
                Some(self.bus.read(mem_addr))
            },
            AddressingMode::AbsoluteJump => panic!("Not implemeneted"),
            AddressingMode::ZeroPage => panic!("Not implemented"),
            AddressingMode::ZeroPageIndexX => panic!("Not implemented"),
            AddressingMode::ZeroPageIndexY => panic!("Not implemented"),
            AddressingMode::AbsoluteIndexX => panic!("Not implemented"),
            AddressingMode::AbsoluteIndexY => panic!("Not implemented"),
            AddressingMode::IndirectX => panic!("Not implemented"),
            AddressingMode::IndirectY => panic!("Not implemented"),
            _ => panic!("Not implemented")
        }
    }


    pub fn run_program(&mut self, program: Vec<u8>) -> Result<(), String> { 
        // This function will take in a program, and execute it step by step
        // TODO: result is Result<()), String> right now, need to change to something more descriptive

        self.program_counter = 0;
        loop {
            // 1. Read opcode and decode it to an instruction, always takes 1 cycle
            let (instruction, addressing_mode) = if let Some(opcode_raw) = program.get(self.program_counter as usize) {
                self.decode_opcode(*opcode_raw)
            } else {
                return Err("Didn't expect instruction".to_string());
            };

            // 2. Read some number of bytes depending on what the addressing mode is and decode the instruction parameter, may take 0-2 cycles
            // Ref: http://www.6502.org/tutorials/6502opcodes.html
            let parameter = self.read_arg(&addressing_mode, &program);

            // 3. Execute the instruction
            // self.execute_instruction(instruction, parameter);

            // 4. Check if program is done, if done
            if program.get(self.program_counter as usize) == None {
                ()
            }
        }
    }


    pub fn execute_instruction(&mut self, instruction: Instruction, parameter: u8) {
        match instruction {
            Instruction::ADC => self.adc(parameter),
            Instruction::LDA => self.lda(parameter),
            Instruction::TAX => self.tax(),
            _ => panic!("Not implemented")
        }
    }

    fn lda(&mut self, parameter: u8) {
        self.reg_a = parameter
    }

    fn ldx(&mut self, parameter: u8) {
        self.reg_x = parameter
    }

    fn ldy(&mut self, parameter: u8) {
        self.reg_y = parameter
    }

    fn tax(&mut self) {
        self.reg_x = self.reg_a
    }

    fn inx(&mut self) {
        self.reg_x += 1
    }

    fn iny(&mut self) {
        self.reg_y += 1;
    }

    fn adc(&mut self, parameter: u8) {
        /// Currently, carrying_add is unstable, so casting to u16 and checking the bit is actually the simplest option
        // let (output, carry) = self.reg_a.carrying_add(parameter, self.status.contains(CpuStatus::CARRY));

        // Cast all relevant values to u16
        let reg_a = self.reg_a as u16;
        let val = parameter as u16;
        let carry = self.status.contains(CpuStatus::CARRY) as u16;

        // Add them together
        let sum = reg_a + val + carry;

        // Keep only least significant byte for result
        let result = sum as u8;

        // Check for carry, maybe refactorable?
        if sum > 0xFF {
            self.status.insert(CpuStatus::CARRY);
        } else {
            self.status.remove(CpuStatus::CARRY);
        }


        // Check for overflow, maybe refactorable?
        if (parameter ^ result) & (self.reg_a ^ result) & 0b1000_0000 != 0 {
            self.status.insert(CpuStatus::OVERFLOW);
        } else {
            self.status.remove(CpuStatus::OVERFLOW);
        }

        // TODO: set flags
        // Set accumulator
        self.reg_a = result;


    }
}

#[cfg(test)]
mod tests {

    
}