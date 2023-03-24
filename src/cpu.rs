use core::panic;
use std::{ops::Add, slice::Iter, fmt::Display};
use crate::bus::Bus;

use bitflags::bitflags;
use log::info;

pub fn some_function() -> i32 {
    // this is just an example function for testing
    5
}

#[derive(Debug)]
pub enum Instruction { // Reorder these at some point to something more logical
    ADC,
    AND,
    ASL,
    BIT,
    // Branching instructions
    BPL,
    BMI,
    BVC,
    BVS,
    BCC,
    BCS,
    BNE,
    BEQ,

    BRK,
    CMP,
    CPX,
    CPY,
    DEC,
    EOR,
    // Flag instructions
    CLC,
    SEC,
    CLI,
    SEI,
    CLV,
    CLD,
    SED,

    INC,
    JMP,
    JSR,
    LDA,
    LDX,
    LDY,
    LSR,
    NOP,
    ORA,
    
    // Register instructions
    TAX,
    TXA,
    DEX,
    INX,
    TAY,
    TYA,
    DEY,
    INY,

    ROL,
    ROR,
    RTI,
    RTS,
    SBC,
    STA,

    // Stack instructions
    TXS,
    TSX,
    PHA,
    PLA,
    PHP,
    PLP,

    STX,
    STY,
}

#[derive(Debug)]
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
    ZeroPageIndexY,
    AbsoluteIndexX,     // val = peek(arg16 + X)
    AbsoluteIndexY,     // val = peek(arg16 + Y)
    IndirectX,          // val = peek(peek((arg + X) % 256) + PEEK((arg + X + 1) % 256) * 256)
    IndirectY,          
}

#[derive(Debug)]
pub enum Param {    // used by an instruction
    Value(u8),
    Address(u16),
}
// NOTE: all cpu opcodes are a single u8 of the form AAABBBCC in binary, BBB defines the addressing mode

bitflags! {
    #[derive(Debug)]
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
    pub fn decode_opcode(&self, opcode: u8) -> (Instruction, AddressingMode) {
        /// Used this reference for decoding opcodes to instruction addressing mode pairs
        /// Ref: http://www.6502.org/tutorials/6502opcodes.html#LDA
        println!("Received opcode {opcode:02x}");
        let result = match opcode {
            // Immediate     ADC #$44      $69  2   2
            // Zero Page     ADC $44       $65  2   3
            // Zero Page,X   ADC $44,X     $75  2   4
            // Absolute      ADC $4400     $6D  3   4
            // Absolute,X    ADC $4400,X   $7D  3   4+
            // Absolute,Y    ADC $4400,Y   $79  3   4+
            // Indirect,X    ADC ($44,X)   $61  2   6
            // Indirect,Y    ADC ($44),Y   $71  2   5+
            0x69 => (Instruction::ADC, AddressingMode::Immediate),
            0x65 => (Instruction::ADC, AddressingMode::ZeroPage),
            0x75 => (Instruction::ADC, AddressingMode::ZeroPageIndexX),
            0x6D => (Instruction::ADC, AddressingMode::Absolute),
            0x7D => (Instruction::ADC, AddressingMode::AbsoluteIndexX),
            0x79 => (Instruction::ADC, AddressingMode::AbsoluteIndexY),
            0x61 => (Instruction::ADC, AddressingMode::IndirectX),
            0x71 => (Instruction::ADC, AddressingMode::IndirectY),
            // Immediate     AND #$44      $29  2   2
            // Zero Page     AND $44       $25  2   3
            // Zero Page,X   AND $44,X     $35  2   4
            // Absolute      AND $4400     $2D  3   4
            // Absolute,X    AND $4400,X   $3D  3   4+
            // Absolute,Y    AND $4400,Y   $39  3   4+
            // Indirect,X    AND ($44,X)   $21  2   6
            // Indirect,Y    AND ($44),Y   $31  2   5+
            0x29 => (Instruction::AND, AddressingMode::Immediate),
            0x25 => (Instruction::AND, AddressingMode::ZeroPage),
            0x35 => (Instruction::AND, AddressingMode::ZeroPageIndexX),
            0x2D => (Instruction::AND, AddressingMode::Absolute),
            0x3D => (Instruction::AND, AddressingMode::AbsoluteIndexX),
            0x39 => (Instruction::AND, AddressingMode::AbsoluteIndexY),
            0x21 => (Instruction::AND, AddressingMode::IndirectX),
            0x31 => (Instruction::AND, AddressingMode::IndirectY),
            // Accumulator   ASL A         $0A  1   2
            // Zero Page     ASL $44       $06  2   5
            // Zero Page,X   ASL $44,X     $16  2   6
            // Absolute      ASL $4400     $0E  3   6
            // Absolute,X    ASL $4400,X   $1E  3   7
            0x0A => (Instruction::ASL, AddressingMode::Accumulator),
            0x06 => (Instruction::ASL, AddressingMode::ZeroPage),
            0x16 => (Instruction::ASL, AddressingMode::ZeroPageIndexX),
            0x0E => (Instruction::ASL, AddressingMode::Absolute),
            0x1E => (Instruction::ASL, AddressingMode::AbsoluteIndexX),
            // BPL (Branch on PLus)           $10
            // BMI (Branch on MInus)          $30
            // BVC (Branch on oVerflow Clear) $50
            // BVS (Branch on oVerflow Set)   $70
            // BCC (Branch on Carry Clear)    $90
            // BCS (Branch on Carry Set)      $B0
            // BNE (Branch on Not Equal)      $D0
            // BEQ (Branch on EQual)          $F0
            0x10 => (Instruction::BPL, AddressingMode::Relative),
            0x30 => (Instruction::BMI, AddressingMode::Relative),
            0x50 => (Instruction::BVC, AddressingMode::Relative),
            0x70 => (Instruction::BVS, AddressingMode::Relative),
            0x90 => (Instruction::BCC, AddressingMode::Relative),
            0xB0 => (Instruction::BCS, AddressingMode::Relative),
            0xD0 => (Instruction::BNE, AddressingMode::Relative),
            0xF0 => (Instruction::BEQ, AddressingMode::Relative),
            // Zero Page     BIT $44       $24  2   3
            // Absolute      BIT $4400     $2C  3   4
            0x24 => (Instruction::BIT, AddressingMode::ZeroPage),
            0x2C => (Instruction::BIT, AddressingMode::Absolute),
            // Implied       BRK           $00  1   7
            0x00 => (Instruction::BRK, AddressingMode::Implicit),
            // Immediate     CMP #$44      $C9  2   2
            // Zero Page     CMP $44       $C5  2   3
            // Zero Page,X   CMP $44,X     $D5  2   4
            // Absolute      CMP $4400     $CD  3   4
            // Absolute,X    CMP $4400,X   $DD  3   4+
            // Absolute,Y    CMP $4400,Y   $D9  3   4+
            // Indirect,X    CMP ($44,X)   $C1  2   6
            // Indirect,Y    CMP ($44),Y   $D1  2   5+
            0xC9 => (Instruction::CMP, AddressingMode::Immediate),
            0xC5 => (Instruction::CMP, AddressingMode::ZeroPage),
            0xD5 => (Instruction::CMP, AddressingMode::ZeroPageIndexX),
            0xCD => (Instruction::CMP, AddressingMode::Absolute),
            0xDD => (Instruction::CMP, AddressingMode::AbsoluteIndexX),
            0xD9 => (Instruction::CMP, AddressingMode::AbsoluteIndexY),
            0xC1 => (Instruction::CMP, AddressingMode::IndirectX),
            0xD1 => (Instruction::CMP, AddressingMode::IndirectY),
            // Immediate     CPX #$44      $E0  2   2
            // Zero Page     CPX $44       $E4  2   3
            // Absolute      CPX $4400     $EC  3   4
            0xE0 => (Instruction::CPX, AddressingMode::Immediate),
            0xE4 => (Instruction::CPX, AddressingMode::ZeroPage),
            0xEC => (Instruction::CPX, AddressingMode::Absolute),
            // Immediate     CPY #$44      $C0  2   2
            // Zero Page     CPY $44       $C4  2   3
            // Absolute      CPY $4400     $CC  3   4
            0xC0 => (Instruction::CPY, AddressingMode::Immediate),
            0xC4 => (Instruction::CPY, AddressingMode::ZeroPage),
            0xCC => (Instruction::CPY, AddressingMode::Absolute),
            // Zero Page     DEC $44       $C6  2   5
            // Zero Page,X   DEC $44,X     $D6  2   6
            // Absolute      DEC $4400     $CE  3   6
            // Absolute,X    DEC $4400,X   $DE  3   7
            0xC6 => (Instruction::DEC, AddressingMode::ZeroPage),
            0xD6 => (Instruction::DEC, AddressingMode::ZeroPageIndexX),
            0xCE => (Instruction::DEC, AddressingMode::Absolute),
            0xDE => (Instruction::DEC, AddressingMode::AbsoluteIndexX),
            // Immediate     EOR #$44      $49  2   2
            // Zero Page     EOR $44       $45  2   3
            // Zero Page,X   EOR $44,X     $55  2   4
            // Absolute      EOR $4400     $4D  3   4
            // Absolute,X    EOR $4400,X   $5D  3   4+
            // Absolute,Y    EOR $4400,Y   $59  3   4+
            // Indirect,X    EOR ($44,X)   $41  2   6
            // Indirect,Y    EOR ($44),Y   $51  2   5+
            0x49 => (Instruction::EOR, AddressingMode::Immediate),
            0x45 => (Instruction::EOR, AddressingMode::ZeroPage),
            0x55 => (Instruction::EOR, AddressingMode::ZeroPageIndexX),
            0x4D => (Instruction::EOR, AddressingMode::Absolute),
            0x5D => (Instruction::EOR, AddressingMode::AbsoluteIndexX),
            0x59 => (Instruction::EOR, AddressingMode::AbsoluteIndexY),
            0x41 => (Instruction::EOR, AddressingMode::IndirectX),
            0x51 => (Instruction::EOR, AddressingMode::IndirectY),
            // CLC (CLear Carry)              $18
            // SEC (SEt Carry)                $38
            // CLI (CLear Interrupt)          $58
            // SEI (SEt Interrupt)            $78
            // CLV (CLear oVerflow)           $B8
            // CLD (CLear Decimal)            $D8
            // SED (SEt Decimal)              $F8
            0x18 => (Instruction::CLC, AddressingMode::Implicit),
            0x38 => (Instruction::SEC, AddressingMode::Implicit),
            0x58 => (Instruction::CLI, AddressingMode::Implicit),
            0x78 => (Instruction::SEI, AddressingMode::Implicit),
            0xB8 => (Instruction::CLV, AddressingMode::Implicit),
            0xD8 => (Instruction::CLD, AddressingMode::Implicit),
            0xF8 => (Instruction::SED, AddressingMode::Implicit),
            // Zero Page     INC $44       $E6  2   5
            // Zero Page,X   INC $44,X     $F6  2   6
            // Absolute      INC $4400     $EE  3   6
            // Absolute,X    INC $4400,X   $FE  3   7
            0xE6 => (Instruction::INC, AddressingMode::ZeroPage),
            0xF6 => (Instruction::INC, AddressingMode::ZeroPageIndexX),
            0xEE => (Instruction::INC, AddressingMode::Absolute),
            0xFE => (Instruction::INC, AddressingMode::AbsoluteIndexX),
            // Absolute      JMP $5597     $4C  3   3
            // Indirect      JMP ($5597)   $6C  3   5
            0x4C => (Instruction::JMP, AddressingMode::AbsoluteJump),
            0x6C => (Instruction::JMP, AddressingMode::IndirectJump),
            // Absolute      JSR $5597     $20  3   6
            0x20 => (Instruction::JSR, AddressingMode::Absolute),
            // Immediate     LDA #$44      $A9  2   2
            // Zero Page     LDA $44       $A5  2   3
            // Zero Page,X   LDA $44,X     $B5  2   4
            // Absolute      LDA $4400     $AD  3   4
            // Absolute,X    LDA $4400,X   $BD  3   4+
            // Absolute,Y    LDA $4400,Y   $B9  3   4+
            // Indirect,X    LDA ($44,X)   $A1  2   6
            // Indirect,Y    LDA ($44),Y   $B1  2   5+
            0xA9 => (Instruction::LDA, AddressingMode::Immediate),
            0xA5 => (Instruction::LDA, AddressingMode::ZeroPage),
            0xB5 => (Instruction::LDA, AddressingMode::ZeroPageIndexX),
            0xAD => (Instruction::LDA, AddressingMode::Absolute),
            0xBD => (Instruction::LDA, AddressingMode::AbsoluteIndexX),
            0xB9 => (Instruction::LDA, AddressingMode::AbsoluteIndexY),
            0xA1 => (Instruction::LDA, AddressingMode::IndirectX),
            0xB1 => (Instruction::LDA, AddressingMode::IndirectY),
            // Immediate     LDX #$44      $A2  2   2
            // Zero Page     LDX $44       $A6  2   3
            // Zero Page,Y   LDX $44,Y     $B6  2   4
            // Absolute      LDX $4400     $AE  3   4
            // Absolute,Y    LDX $4400,Y   $BE  3   4+
            0xA2 => (Instruction::LDX, AddressingMode::Immediate),
            0xA6 => (Instruction::LDX, AddressingMode::ZeroPage),
            0xB6 => (Instruction::LDX, AddressingMode::ZeroPageIndexY),
            0xAE => (Instruction::LDX, AddressingMode::Absolute),
            0xBE => (Instruction::LDX, AddressingMode::AbsoluteIndexY),
            // Immediate     LDY #$44      $A0  2   2
            // Zero Page     LDY $44       $A4  2   3
            // Zero Page,X   LDY $44,X     $B4  2   4
            // Absolute      LDY $4400     $AC  3   4
            // Absolute,X    LDY $4400,X   $BC  3   4+
            0xA0 => (Instruction::LDY, AddressingMode::Immediate),
            0xA4 => (Instruction::LDY, AddressingMode::ZeroPage),
            0xB4 => (Instruction::LDY, AddressingMode::ZeroPageIndexX),
            0xAC => (Instruction::LDY, AddressingMode::Absolute),
            0xBC => (Instruction::LDY, AddressingMode::AbsoluteIndexX),
            // Accumulator   LSR A         $4A  1   2
            // Zero Page     LSR $44       $46  2   5
            // Zero Page,X   LSR $44,X     $56  2   6
            // Absolute      LSR $4400     $4E  3   6
            // Absolute,X    LSR $4400,X   $5E  3   7
            0x4A => (Instruction::LSR, AddressingMode::Accumulator),
            0x46 => (Instruction::LSR, AddressingMode::ZeroPage),
            0x56 => (Instruction::LSR, AddressingMode::ZeroPageIndexX),
            0x4E => (Instruction::LSR, AddressingMode::Absolute),
            0x5E => (Instruction::LSR, AddressingMode::AbsoluteIndexX),
            // Implied       NOP           $EA  1   2
            0xEA => (Instruction::NOP, AddressingMode::Implicit),
            // Immediate     ORA #$44      $09  2   2
            // Zero Page     ORA $44       $05  2   3
            // Zero Page,X   ORA $44,X     $15  2   4
            // Absolute      ORA $4400     $0D  3   4
            // Absolute,X    ORA $4400,X   $1D  3   4+
            // Absolute,Y    ORA $4400,Y   $19  3   4+
            // Indirect,X    ORA ($44,X)   $01  2   6
            // Indirect,Y    ORA ($44),Y   $11  2   5+
            0x09 => (Instruction::ORA, AddressingMode::Immediate),
            0x05 => (Instruction::ORA, AddressingMode::ZeroPage),
            0x15 => (Instruction::ORA, AddressingMode::ZeroPageIndexX),
            0x0D => (Instruction::ORA, AddressingMode::Absolute),
            0x1D => (Instruction::ORA, AddressingMode::AbsoluteIndexX),
            0x19 => (Instruction::ORA, AddressingMode::AbsoluteIndexY),
            0x01 => (Instruction::ORA, AddressingMode::IndirectX),
            0x11 => (Instruction::ORA, AddressingMode::IndirectY),
            // TAX (Transfer A to X)    $AA
            // TXA (Transfer X to A)    $8A
            // DEX (DEcrement X)        $CA
            // INX (INcrement X)        $E8
            // TAY (Transfer A to Y)    $A8
            // TYA (Transfer Y to A)    $98
            // DEY (DEcrement Y)        $88
            // INY (INcrement Y)        $C8
            0xAA => (Instruction::TAX, AddressingMode::Implicit),
            0x8A => (Instruction::TXA, AddressingMode::Implicit),
            0xCA => (Instruction::DEX, AddressingMode::Implicit),
            0xE8 => (Instruction::INX, AddressingMode::Implicit),
            0xA8 => (Instruction::TAY, AddressingMode::Implicit),
            0x98 => (Instruction::TYA, AddressingMode::Implicit),
            0x88 => (Instruction::DEY, AddressingMode::Implicit),
            0xC8 => (Instruction::INY, AddressingMode::Implicit),
            // Accumulator   ROL A         $2A  1   2
            // Zero Page     ROL $44       $26  2   5
            // Zero Page,X   ROL $44,X     $36  2   6
            // Absolute      ROL $4400     $2E  3   6
            // Absolute,X    ROL $4400,X   $3E  3   7
            0x2A => (Instruction::ROL, AddressingMode::Accumulator),
            0x26 => (Instruction::ROL, AddressingMode::ZeroPage),
            0x36 => (Instruction::ROL, AddressingMode::ZeroPageIndexX),
            0x2E => (Instruction::ROL, AddressingMode::Absolute),
            0x3E => (Instruction::ROL, AddressingMode::AbsoluteIndexX),
            // Accumulator   ROR A         $6A  1   2
            // Zero Page     ROR $44       $66  2   5
            // Zero Page,X   ROR $44,X     $76  2   6
            // Absolute      ROR $4400     $6E  3   6
            // Absolute,X    ROR $4400,X   $7E  3   7
            0x6A => (Instruction::ROR, AddressingMode::Accumulator),
            0x66 => (Instruction::ROR, AddressingMode::ZeroPage),
            0x76 => (Instruction::ROR, AddressingMode::ZeroPageIndexX),
            0x6E => (Instruction::ROR, AddressingMode::Absolute),
            0x7E => (Instruction::ROR, AddressingMode::AbsoluteIndexX),
            // Implied       RTI           $40  1   6
            0x40 => (Instruction::RTI, AddressingMode::Implicit),
            // Implied       RTS           $60  1   6
            0x60 => (Instruction::RTS, AddressingMode::Implicit),
            // Immediate     SBC #$44      $E9  2   2
            // Zero Page     SBC $44       $E5  2   3
            // Zero Page,X   SBC $44,X     $F5  2   4
            // Absolute      SBC $4400     $ED  3   4
            // Absolute,X    SBC $4400,X   $FD  3   4+
            // Absolute,Y    SBC $4400,Y   $F9  3   4+
            // Indirect,X    SBC ($44,X)   $E1  2   6
            // Indirect,Y    SBC ($44),Y   $F1  2   5+
            0xE9 => (Instruction::SBC, AddressingMode::Immediate),
            0xE5 => (Instruction::SBC, AddressingMode::ZeroPage),
            0xF5 => (Instruction::SBC, AddressingMode::ZeroPageIndexX),
            0xED => (Instruction::SBC, AddressingMode::Absolute),
            0xFD => (Instruction::SBC, AddressingMode::AbsoluteIndexX),
            0xF9 => (Instruction::SBC, AddressingMode::AbsoluteIndexY),
            0xE1 => (Instruction::SBC, AddressingMode::IndirectX),
            0xF1 => (Instruction::SBC, AddressingMode::IndirectY),
            // Zero Page     STA $44       $85  2   3
            // Zero Page,X   STA $44,X     $95  2   4
            // Absolute      STA $4400     $8D  3   4
            // Absolute,X    STA $4400,X   $9D  3   5
            // Absolute,Y    STA $4400,Y   $99  3   5
            // Indirect,X    STA ($44,X)   $81  2   6
            // Indirect,Y    STA ($44),Y   $91  2   6
            0x85 => (Instruction::STA, AddressingMode::ZeroPage),
            0x95 => (Instruction::STA, AddressingMode::ZeroPageIndexX),
            0x8D => (Instruction::STA, AddressingMode::Absolute),
            0x9D => (Instruction::STA, AddressingMode::AbsoluteIndexX),
            0x99 => (Instruction::STA, AddressingMode::AbsoluteIndexY),
            0x81 => (Instruction::STA, AddressingMode::IndirectX),
            0x91 => (Instruction::STA, AddressingMode::IndirectY),
            // TXS (Transfer X to Stack ptr)   $9A  2
            // TSX (Transfer Stack ptr to X)   $BA  2
            // PHA (PusH Accumulator)          $48  3
            // PLA (PuLl Accumulator)          $68  4
            // PHP (PusH Processor status)     $08  3
            // PLP (PuLl Processor status)     $28  4
            0x9A => (Instruction::TXS, AddressingMode::Implicit),
            0xBA => (Instruction::TSX, AddressingMode::Implicit),
            0x48 => (Instruction::PHA, AddressingMode::Implicit),
            0x68 => (Instruction::PLA, AddressingMode::Implicit),
            0x08 => (Instruction::PHP, AddressingMode::Implicit),
            0x28 => (Instruction::PLP, AddressingMode::Implicit),
            // Zero Page     STX $44       $86  2   3
            // Zero Page,Y   STX $44,Y     $96  2   4
            // Absolute      STX $4400     $8E  3   4
            0x86 => (Instruction::STX, AddressingMode::ZeroPage),
            0x96 => (Instruction::STX, AddressingMode::ZeroPageIndexY),
            0x8E => (Instruction::STX, AddressingMode::Absolute),
            // Zero Page     STY $44       $84  2   3
            // Zero Page,X   STY $44,X     $94  2   4
            // Absolute      STY $4400     $8C  3   4
            0x84 => (Instruction::STY, AddressingMode::ZeroPage),
            0x94 => (Instruction::STY, AddressingMode::ZeroPageIndexX),
            0x8C => (Instruction::STY, AddressingMode::Absolute),
            _ => panic!("Opcode not implemented {}", opcode)
        };
        println!("Decoded opcode to {result:?}");
        result
    }
}



impl CPU {
    
    pub fn new() -> Self {
        CPU {
            reg_a: 0,
            reg_x: 0,
            reg_y: 0,
            status: CpuStatus::ALWAYS | CpuStatus::INT_DISABLE,
            stack_pointer: 0,      // probably needs to initialize to something else
            program_counter: 0,      // same here
            bus: Bus::new(),
        }
    }

    fn read_arg(&mut self, mode: &AddressingMode, program: &Vec<u8>) -> Option<Param> {
        /// Based on the addressing mode, read `n` number of argument bytes from the program and process it into a parameter
        /// to be used by some instruction
        println!("Reading argument...");
        match mode {
            AddressingMode::Implicit => None,
            AddressingMode::Accumulator => {
                Some(Param::Value(self.reg_a))
            },
            AddressingMode::Immediate | AddressingMode::Relative => {
                Some(Param::Value(self.read_byte(&program)?))
            },
            AddressingMode::IndirectJump => {
                // first read two bytes
                let mem_addr = self.read_two_bytes(&program)?;
                // read the two bytes from memory and form it into a mem addr
                let lsb = self.bus.read(mem_addr) as u16;
                let msb = self.bus.read(mem_addr + 1) as u16;
                let mem_addr = (msb << 8) + lsb;
                // now read from memory
                Some(Param::Address(mem_addr))
            },
            AddressingMode::Absolute => {
                // first read two bytes
                let mem_addr = self.read_two_bytes(&program)?;
                // read memory from bus
                Some(Param::Address(mem_addr))
            },
            AddressingMode::AbsoluteJump => panic!("Not implemeneted"),
            AddressingMode::ZeroPage => {
                // read single byte, msb is always 0x00
                let zero_page_addr = self.read_byte(&program)? as u16;
                // read memory from bus
                Some(Param::Address(zero_page_addr))
            },
            AddressingMode::ZeroPageIndexX => {
                let zero_page_addr = self.read_byte(&program)?.wrapping_add(self.reg_x) as u16;
                Some(Param::Address(zero_page_addr))
            },
            AddressingMode::ZeroPageIndexY => {
                let zero_page_addr = self.read_byte(&program)?.wrapping_add(self.reg_y) as u16;
                Some(Param::Address(zero_page_addr))
            },
            AddressingMode::AbsoluteIndexX => {
                // Form <instruction> <addr>, X where <addr> is u16, specifies the value of read(<addr> + 1)
                let mem_addr = self.read_two_bytes(&program)?.wrapping_add(self.reg_x as u16);
                Some(Param::Address(mem_addr))
            },
            AddressingMode::AbsoluteIndexY => {
                // Same as AbsoluteIndexX, but with reg_y instead
                let mem_addr = self.read_two_bytes(&program)?.wrapping_add(self.reg_y as u16);
                Some(Param::Address(mem_addr))
            },
            AddressingMode::IndirectX => {
                // Form <instruction (<addr>, X), where <addr> is u8
                let zero_page_addr = (self.read_byte(&program)?.wrapping_add(self.reg_x)) as u16;

                let lsb = self.bus.read(zero_page_addr) as u16;
                let msb = self.bus.read(zero_page_addr + 1) as u16;
                
                let indirect = (msb << 8) + lsb;
                // read memory from bus
                Some(Param::Address(indirect))
            },
            AddressingMode::IndirectY => {
                let zero_page_addr = self.read_byte(&program)? as u16;

                let lsb = self.bus.read(zero_page_addr) as u16;
                let msb = self.bus.read(zero_page_addr + 1) as u16;

                let indirect = (msb << 8) + lsb + (self.reg_y as u16);

                Some(Param::Address(indirect))
            },
        }
    }

    fn read_byte(&mut self, program: &Vec<u8>) -> Option<u8> {
        println!("Read byte at {:02x}", self.program_counter);
        let byte = *program.get(self.program_counter as usize)?;
        self.program_counter += 1;
        Some(byte)
    }

    fn read_two_bytes(&mut self, program: &Vec<u8>) -> Option<u16> {
        println!("Read bytes at {:02x}, {:02x}", self.program_counter, self.program_counter + 1);
        let lsb = *program.get(self.program_counter as usize)? as u16;
        let msb = *program.get((self.program_counter + 1) as usize)? as u16;
        self.program_counter += 2;

        let two_bytes = (msb << 8) + lsb;
        Some(two_bytes)
    }


    pub fn run_program(&mut self, program: Vec<u8>) -> Result<(), String> { 
        // This function will take in a program, and execute it step by step
        // TODO: result is Result<()), String> right now, need to change to something more descriptive

        self.program_counter = 0;
        loop {
            // 1. Read opcode and decode it to an instruction, always takes 1 cycle
            let (instruction, addressing_mode) = if let Some(opcode_raw) = self.read_byte(&program) {
                self.decode_opcode(opcode_raw)
            } else {
                return Err("Didn't expect instruction".to_string());
            };

            // 2. Read some number of bytes depending on what the addressing mode is and decode the instruction parameter, may take many cycles
            // Ref: http://www.6502.org/tutorials/6502opcodes.html
            let parameter = self.read_arg(&addressing_mode, &program);

            // 3. Execute the instruction
            self.execute_instruction(instruction, parameter);

            // 4. Check if program is done, if done return TODO turn to check brk flag
            if program.get(self.program_counter as usize) == None {
                self.status.insert(CpuStatus::BRK);
                ()
            }
        }
    }


    pub fn execute_instruction(&mut self, instruction: Instruction, parameter: Option<Param>) {
        match instruction {
            // Instruction::ADC => self.adc(parameter),
            Instruction::LDA => {
                match parameter {
                    Some(Param::Value(val)) => self.lda(val),
                    Some(Param::Address(mem_addr)) => self.lda(self.bus.read(mem_addr)),
                    _ => panic!("Invalid parameter"),
                }
            },
            // Instruction::TAX => self.tax(),
            Instruction::STA => {
                match parameter {
                    Some(Param::Address(mem_addr)) => self.sta(mem_addr),
                    _ => panic!("Invalid parameter"),
                }
            },
            _ => panic!("Not implemented"),
        }
    }

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

        // Check negative flag
        if result & 0b1000_0000 != 0 {
            self.status.insert(CpuStatus::NEGATIVE);
        } else {
            self.status.remove(CpuStatus::NEGATIVE);
        }
        // Check overflow flag
        if (parameter ^ result) & (self.reg_a ^ result) & 0b1000_0000 != 0 {
            self.status.insert(CpuStatus::OVERFLOW);
        } else {
            self.status.remove(CpuStatus::OVERFLOW);
        }
        // Check zero flag
        if result == 0 {
            self.status.insert(CpuStatus::ZERO);
        } else {
            self.status.remove(CpuStatus::ZERO);
        }
        // Check carry flag
        if sum > 0xFF {
            self.status.insert(CpuStatus::CARRY);
        } else {
            self.status.remove(CpuStatus::CARRY);
        }
        // Set accumulator
        self.reg_a = result;
    }

    fn lda(&mut self, parameter: u8) {
        // Affects Flags: N Z
        self.reg_a = parameter;

        // check zero flag
        if self.reg_a == 0 {
            self.status.insert(CpuStatus::ZERO);
        } else {
            self.status.remove(CpuStatus::ZERO);
        }
        // check neg flag
        if self.reg_a & 0b1000_0000 != 0 {
            self.status.insert(CpuStatus::NEGATIVE);
        } else {
            self.status.remove(CpuStatus::NEGATIVE);
        }
    }

    fn ldx(&mut self, parameter: u8) {
        // Affects Flags: N Z
        self.reg_x = parameter;

        // check zero flag
        if self.reg_x == 0 {
            self.status.insert(CpuStatus::ZERO);
        } else {
            self.status.remove(CpuStatus::ZERO);
        }
        // check neg flag
        if self.reg_x & 0b1000_0000 != 0 {
            self.status.insert(CpuStatus::NEGATIVE);
        } else {
            self.status.remove(CpuStatus::NEGATIVE);
        }
    }

    fn ldy(&mut self, parameter: u8) {
        // Affects Flags: N Z
        self.reg_y = parameter;

        // check zero flag
        if self.reg_y == 0 {
            self.status.insert(CpuStatus::ZERO);
        } else {
            self.status.remove(CpuStatus::ZERO);
        }
        // check neg flag
        if self.reg_y & 0b1000_0000 != 0 {
            self.status.insert(CpuStatus::NEGATIVE);
        } else {
            self.status.remove(CpuStatus::NEGATIVE);
        }
    }

    fn tax(&mut self) {
        // Affects Flags: N Z
        self.reg_x = self.reg_a;

        // check zero flag
        if self.reg_x == 0 {
            self.status.insert(CpuStatus::ZERO);
        } else {
            self.status.remove(CpuStatus::ZERO);
        }
        // check neg flag
        if self.reg_x & 0b1000_0000 != 0 {
            self.status.insert(CpuStatus::NEGATIVE);
        } else {
            self.status.remove(CpuStatus::NEGATIVE);
        }

    }

    fn inx(&mut self) {
        self.reg_x += 1
    }

    fn iny(&mut self) {
        self.reg_y += 1;
    }

    fn sta(&mut self, address: u16) {
        self.bus.write(address, self.reg_a)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    pub fn test_lda_sta() {
        let mut cpu = CPU::new();
        // $0600    a9 01     LDA #$01
        // $0602    8d 00 02  STA $0200
        // $0605    a9 05     LDA #$05
        // $0607    8d 01 02  STA $0201
        // $060a    a9 08     LDA #$08
        // $060c    8d 02 02  STA $0202

        // Expected
        // A=$08 X=$00 Y=$00
        // SP=$ff PC=$0613
        // NV-BDIZC
        // 00110000
        let program = vec![
            0xA9, 0x01, 
            0x8D, 0x00, 0x02,
            0xA9, 0x05,
            0x8D, 0x01, 0x02,
            0xA9, 0x08,
            0x8D, 0x02, 0x02,
        ];

        cpu.run_program(program);

        // assert registers
        assert_eq!(0x08, cpu.reg_a);
        assert_eq!(0x00, cpu.reg_x);
        assert_eq!(0x00, cpu.reg_y);
        // assert status
        assert!(cpu.status.contains(CpuStatus::ALWAYS | CpuStatus::BRK));
        // assert memory
        assert_eq!(
            [cpu.bus.read(0x200), cpu.bus.read(0x201), cpu.bus.read(0x202)], 
            [0x01, 0x05, 0x08]
        )

    }
}