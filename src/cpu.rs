use core::panic;
use std::ops::Add;

pub fn some_function() -> i32 {
    // this is just an example function for testing
    5
}

pub enum Instruction {
    ADC,

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

pub struct CPU {
    // General purpose registers
    pub reg_a: u8,
    pub reg_x: u8,
    pub reg_y: u8,
    // Special purpose registers
    status: u8,
    stack_pointer: u16,
    program_counter: u8,
}

const ADDRESSING_MODE_MASK: u8 = 0b0001_1100;
const INSTRUCTION_MASK: u8 = 0b1110_0011;

impl CPU {
    
    pub fn new() -> Self {
        CPU {
            reg_a: 0,
            reg_x: 0,
            reg_y: 0,
            status: 0,
            stack_pointer: 0,      // probably needs to initialize to something else
            program_counter: 0,      // same here
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


    fn decode_arg(&self, arg: u8, mode: &AddressingMode) -> u16 {
        /// Decodes an argument byte and the addressing mode into the proper value

        match mode {
            AddressingMode::Implicit => 0,
            AddressingMode::Accumulator => self.reg_a as u16,
            AddressingMode::Immediate => arg as u16,

            AddressingMode::IndirectJump => panic!("Not implemented"),
            AddressingMode::Relative => panic!("Not implemented"),
            AddressingMode::Absolute => arg as u16,
            AddressingMode::AbsoluteJump => panic!("Not implemeneted"),
            AddressingMode::ZeroPage => panic!("Not implemented"),
            AddressingMode::ZeroPageIndexX => panic!("Not implemented"),
            AddressingMode::ZeroPageIndexY => panic!("Not implemented"),
            AddressingMode::AbsoluteIndexX => panic!("Not implemented"),
            AddressingMode::AbsoluteIndexY => panic!("Not implemented"),
            AddressingMode::IndirectX => panic!("Not implemented"),
            AddressingMode::IndirectY => panic!("Not implemented"),
        }
    }


    pub fn run_program(&mut self, program: Vec<u8>) {
        // This function will take in a program, and execute it step by step, the program is read back to front

        self.program_counter = 0;
        let mut program_iter = program.iter();
        loop {
            // 1. Read opcode and decode it to an instruction
            let (instruction, addressing_mode) = if let Some(opcode_raw) = program_iter.next() {
                self.decode_opcode(*opcode_raw)
            } else {
                break
            };

            
            // 2. Read argument and decode
            let parameter = if let Some(arg) = program_iter.next() {
                self.decode_arg(*arg, &addressing_mode)
            } else {
                break;
            };

            // 3. Read an extra byte if in Absolute mode

            let parameter = match addressing_mode {
                AddressingMode::Absolute | AddressingMode::AbsoluteIndexX | AddressingMode::AbsoluteIndexY => {
                    match program_iter.next() {
                        Some(arg) => (parameter << 8) + (*arg as u16),
                        None => break
                    }
                }
                _ => parameter
            };
            

            // 4. Execute the instruction


            break;
        }
    }


    pub fn execute_instruction(&mut self, instruction: Instruction, parameter: u16) {
        match instruction {
            Instruction::ADC => {
                
            }
            _ => panic!("Not implemented")
        }
    }


    pub fn execute_adc(&mut self, mode: AddressingMode) {
        

        // parameter.
    }
}
