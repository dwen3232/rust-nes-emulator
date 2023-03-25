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
    ADC, AND, ASL, BIT,
    // Branching instructions
    BPL, BMI, BVC, BVS, BCC, BCS, BNE, BEQ, BRK,
    CMP, CPX, CPY, DEC, EOR,
    // Flag instructions
    CLC, SEC, CLI, SEI, CLV, CLD, SED,
    INC, JMP, JSR, LDA, LDX, LDY, LSR, NOP, ORA,
    // Register instructions
    TAX, TXA, DEX, INX, TAY, TYA, DEY, INY, ROL, ROR, RTI, RTS, SBC,
    // Stack instructions
    TXS, TSX, PHA, PLA, PHP, PLP,
    STA, STX, STY,
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



const STACK_POINTER_START: u8 = 0xff;
const PROGRAM_COUNTER_START: u16 = 0x600;

#[derive(Debug)]
pub struct CPU {
    // General purpose registers
    pub reg_a: u8,
    pub reg_x: u8,
    pub reg_y: u8,
    // Special purpose registers
    status: CpuStatus,
    stack_pointer: u8,
    program_counter: u16,

    // Bus
    bus: Bus,
}

impl CPU {
    fn decode_opcode(&self, opcode: u8) -> (Instruction, AddressingMode) {
        // Used this reference for decoding opcodes to instruction addressing mode pairs
        // Ref: http://www.6502.org/tutorials/6502opcodes.html#LDA
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
            _ => panic!("Opcode not implemented {:x}", opcode)
        };
        result
    }

    fn read_arg(&mut self, mode: &AddressingMode, program: &Vec<u8>) -> Option<Param> {
        // Based on the addressing mode, read `n` number of argument bytes from the program and process it into a parameter
        // to be used by some instruction
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
            AddressingMode::AbsoluteJump => {
                let mem_addr = self.read_two_bytes(program)?;
                Some(Param::Address(mem_addr))
            },
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
}

impl CPU {  // Public functions
    pub fn new() -> Self {
        CPU {
            reg_a: 0,
            reg_x: 0,
            reg_y: 0,
            status: CpuStatus::ALWAYS | CpuStatus::BRK,
            stack_pointer: STACK_POINTER_START,      // probably needs to initialize to something else
            program_counter: PROGRAM_COUNTER_START,      // same here
            bus: Bus::new(),
        }
    }
    // pub fn run_program_with_callback(&mut self, program: Vec<u8>) -> Result<(), String> {

    // }

    pub fn run_program(&mut self, program: Vec<u8>) -> Result<(), String> { 
        // This function will take in a program, and execute it step by step
        // TODO: result is Result<()), String> right now, need to change to something more descriptive
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
            // if self.read_byte(&program) == None {
            //     self.status.insert(CpuStatus::BRK);
            //     ()
            // }
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
                        self.adc(self.bus.read(mem_addr)),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::AND => {
                match parameter {
                    Some(Param::Value(val)) => 
                        self.and(val),
                    Some(Param::Address(mem_addr)) => 
                        self.and(self.bus.read(mem_addr)),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::ASL => {
                match parameter {
                    Some(Param::Value(val)) => 
                        self.asl(val),
                    Some(Param::Address(mem_addr)) => 
                        self.asl(self.bus.read(mem_addr)),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::BIT => {
                match parameter {
                    Some(Param::Value(val)) => 
                        self.bit(val),
                    Some(Param::Address(mem_addr)) => 
                        self.bit(self.bus.read(mem_addr)),
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
                        self.cmp(self.bus.read(mem_addr)),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::CPX => {
                match parameter {
                    Some(Param::Value(val)) => 
                        self.cpx(val),
                    Some(Param::Address(mem_addr)) => 
                        self.cpx(self.bus.read(mem_addr)),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::CPY => {
                match parameter {
                    Some(Param::Value(val)) => 
                        self.cpy(val),
                    Some(Param::Address(mem_addr)) => 
                        self.cpy(self.bus.read(mem_addr)),
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
                        self.eor(self.bus.read(mem_addr)),
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
                        self.lda(self.bus.read(mem_addr)),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::LDX => {
                match parameter {
                    Some(Param::Value(val)) => 
                        self.ldx(val),
                    Some(Param::Address(mem_addr)) => 
                        self.ldx(self.bus.read(mem_addr)),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::LDY => {
                match parameter {
                    Some(Param::Value(val)) => 
                        self.ldy(val),
                    Some(Param::Address(mem_addr)) => 
                        self.ldy(self.bus.read(mem_addr)),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::LSR => {
                match parameter {
                    Some(Param::Value(val)) => 
                        self.lsr(val),
                    Some(Param::Address(mem_addr)) => 
                        self.lsr(self.bus.read(mem_addr)),
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
                        self.ora(self.bus.read(mem_addr)),
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
                        self.rol(val),
                    Some(Param::Address(mem_addr)) => 
                        self.rol(self.bus.read(mem_addr)),
                    _ => panic!("Invalid parameter"),
                }
            },
            Instruction::ROR => {
                match parameter {
                    Some(Param::Value(val)) => 
                        self.ror(val),
                    Some(Param::Address(mem_addr)) => 
                        self.ror(self.bus.read(mem_addr)),
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
                        self.sbc(self.bus.read(mem_addr)),
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
                        self.txs(),
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
            // _ => panic!("Not implemented"),
        }
    }
}

impl CPU {  // helper functions
    fn read_byte(&mut self, program: &Vec<u8>) -> Option<u8> {
        // must increment program counter before the attempted read returns None
        let read_addr = self.program_counter;
        self.program_counter += 1;
        let byte = *program.get((read_addr - PROGRAM_COUNTER_START) as usize)?;
        Some(byte)
    }

    fn read_two_bytes(&mut self, program: &Vec<u8>) -> Option<u16> {
        let lsb = self.read_byte(program)? as u16;
        let msb = self.read_byte(program)? as u16;
        let two_bytes = (msb << 8) + lsb;
        Some(two_bytes)
    }

    fn push_to_stack(&mut self, value: u8) {
        // stack located from 0x100 to 0x1FF, growing downward
        let stack_addr = (self.stack_pointer as u16) + 0x100;
        self.stack_pointer -= 1;
        self.bus.write(stack_addr, value);
    }

    fn pop_from_stack(&mut self) -> u8 {
        let stack_addr = (self.stack_pointer as u16) + 0x100;
        self.stack_pointer += 1;
        self.bus.read(stack_addr)
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

    fn asl(&mut self, parameter: u8) {
        // Affects Flags: N Z C

        let result = (parameter as u16) << 1;
        self.reg_a = result as u8;

        self.set_negative_flag(self.reg_a);
        self.set_zero_flag(self.reg_a);
        self.set_carry_flag(result);
    }

    fn bit(&mut self, parameter: u8) {
        // Affects Flags: N V Z
        let result = self.reg_a & parameter;

        self.set_negative_flag(parameter); // neg if bit 7 in param is 1
        
        if result & 0b0100_0000 != 0 {
            self.status.insert(CpuStatus::OVERFLOW);
        } else {
            self.status.remove(CpuStatus::OVERFLOW);
        }

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
        let result = self.bus.read(address).wrapping_sub(1);
        self.bus.write(address, result);

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
        let result = self.bus.read(address).wrapping_add(1);
        self.bus.write(address, result);

        self.set_negative_flag(result);
        self.set_zero_flag(result);
    }
    
    fn jmp(&mut self, address: u16) {
        // Affects Flags: None
        self.program_counter = address;
    }

    fn jsr(&mut self, address: u16) {
        // Affects Flags: None
        let address = address - 1;
        let lsb = address as u8;
        let msb = (address >> 8) as u8;
        // Push msb first
        self.push_to_stack(msb);
        self.push_to_stack(lsb);
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

    fn lsr(&mut self, parameter: u8) {
        // Affects Flags: N Z C
        // I think this writes to reg_a? Not sure
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
        self.reg_y = self.reg_x.wrapping_sub(1);

        self.set_negative_flag(self.reg_y);
        self.set_zero_flag(self.reg_y);
    }

    fn iny(&mut self) {
        // Affects Flags: N Z
        self.reg_y = self.reg_x.wrapping_add(1);

        self.set_negative_flag(self.reg_y);
        self.set_zero_flag(self.reg_y);
    }

    fn rol(&mut self, parameter: u8) {
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

    fn ror(&mut self, parameter: u8) {
        // Affects Flags: N Z C
        let mut result = parameter >> 1;
        if self.status.contains(CpuStatus::CARRY) {
            result += 0b1000_000;
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
        self.push_to_stack(self.status.bits());
    }

    fn plp(&mut self) {
        // Affects Flags: All
        self.status = CpuStatus::from_bits(self.pop_from_stack()).unwrap();
        // plp ALWAYS sets BRK flag
        self.status.insert(CpuStatus::BRK);
    }

    fn sta(&mut self, address: u16) {
        // Affected Flags: None
        self.bus.write(address, self.reg_a);
    }

    fn stx(&mut self, address: u16) {
        // Affected Flags: None
        self.bus.write(address, self.reg_x);
    }

    fn sty(&mut self, address: u16) {
        // Affected Flags: None
        self.bus.write(address, self.reg_y);
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    pub fn test_lda_sta() {
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
        let mut cpu = CPU::new();
        let program = vec![
            0xA9, 0x01, 
            0x8D, 0x00, 0x02,
            0xA9, 0x05,
            0x8D, 0x01, 0x02,
            0xA9, 0x08,
            0x8D, 0x02, 0x02,
        ];

        cpu.run_program(program);

        assert_eq!(0x08, cpu.reg_a, "Register A: {:x}", cpu.reg_a);
        assert_eq!(0x00, cpu.reg_x, "Register X: {:x}", cpu.reg_x);
        assert_eq!(0x00, cpu.reg_y, "Register Y: {:x}", cpu.reg_y);
        assert_eq!(0xff, cpu.stack_pointer, "Stack Pointer: {:x}", cpu.stack_pointer);
        // assert_eq!(0x613, cpu.program_counter, "Program Counter: {:x}", cpu.program_counter);
        // assert memory
        assert_eq!(
            [cpu.bus.read(0x200), cpu.bus.read(0x201), cpu.bus.read(0x202)], 
            [0x01, 0x05, 0x08]
        );
    }

    #[test]
    pub fn test_lda_tax_inx_adc_brk() {
        // $0600    a9 c0     LDA #$c0
        // $0602    aa        TAX 
        // $0603    e8        INX 
        // $0604    69 c4     ADC #$c4
        // $0606    00        BRK 

        // Expected
        // A=$84 X=$c1 Y=$00
        // SP=$ff PC=$060a
        // NV-BDIZC
        // 10110001
        let mut cpu = CPU::new();
        let program = vec![
            0xA9, 0xC0,
            0xAA,
            0xE8,
            0x69, 0xC4,
            0x00,
        ];

        cpu.run_program(program);

        // assert registers
        assert_eq!(0x84, cpu.reg_a);
        assert_eq!(0xc1, cpu.reg_x);
        assert_eq!(0x00, cpu.reg_y);
        // assert status
        assert!(cpu.status.contains(
            CpuStatus::NEGATIVE | CpuStatus::ALWAYS | CpuStatus::BRK
        ));
    }

    #[test]
    pub fn test_adc_overflow_1() {
        // $0600    18        CLC 
        // $0601    a9 7f     LDA #$7f
        // $0603    69 01     ADC #$01

        // Expected
        // A=$80 X=$00 Y=$00
        // SP=$ff PC=$0606
        // NV-BDIZC
        // 11110000

        let mut cpu = CPU::new();
        let program = vec![
            0x18,
            0xA9, 0x7f,
            0x69, 0x01,
        ];

        cpu.run_program(program);

         // assert registers
        assert_eq!(0x80, cpu.reg_a);
        assert_eq!(0x00, cpu.reg_x);
        assert_eq!(0x00, cpu.reg_y);
        assert!(cpu.status.contains(
            CpuStatus::from_bits(0b11110000).unwrap()
        ), "CPU Status: {:b}", cpu.status.bits())
    }

    #[test]
    pub fn test_cpx_1() {
        // $0600    a2 08     LDX #$08
        // $0602    ca        DEX 
        // $0603    8e 00 02  STX $0200
        // $0606    e0 03     CPX #$03
        // $060a    8e 01 02  STX $0201
        
        //
        // A=$00 X=$00 Y=$00
        // SP=$ff PC=$0600
        // NV-BDIZC
        // 00110000
        let mut cpu = CPU::new();
        let program = vec![
            0xa2, 0x08,
            0xca,
            0x8e, 0x00, 0x02,
            0xe0, 0x03,
            0x8e, 0x01, 0x02,
        ];

        cpu.run_program(program);

         // assert registers
        assert_eq!(0x00, cpu.reg_a, "Register A: {:x}", cpu.reg_a);
        assert_eq!(0x07, cpu.reg_x, "Register X: {:x}", cpu.reg_x);
        assert_eq!(0x00, cpu.reg_y, "Register Y: {:x}", cpu.reg_y);
        assert_eq!(0xff, cpu.stack_pointer, "Stack Pointer: {:x}", cpu.stack_pointer);
        // assert_eq!(0x609, cpu.program_counter, "Program Counter: {:x}", cpu.program_counter);
        assert!(cpu.status.contains(
            CpuStatus::from_bits(0b00110001).unwrap()
        ), "CPU Status: {:b}", cpu.status.bits());

        assert_eq!(
            [cpu.bus.read(0x200), cpu.bus.read(0x201), cpu.bus.read(0x202)], 
            [0x07, 0x7, 0x00]
        );
    }

    #[test]
    pub fn test_bne_1() {
        // $0600    a2 08     LDX #$08
        // $0602    ca        DEX 
        // $0603    8e 00 02  STX $0200
        // $0606    e0 03     CPX #$03
        // $0608    d0 f8     BNE $0602
        // $060a    8e 01 02  STX $0201
        // $060d    00        BRK 
        
        // Expected
        // A=$00 X=$03 Y=$00
        // SP=$ff PC=$060e
        // NV-BDIZC
        // 00110011
        let mut cpu = CPU::new();
        let program = vec![
            0xa2, 0x08,
            0xca,
            0x8e, 0x00, 0x02,
            0xe0, 0x03,
            0xd0, 0xf8,
            0x8e, 0x01, 0x02,
            0x00,
        ];

        cpu.run_program(program);

         // assert registers
        assert_eq!(0x00, cpu.reg_a, "Register A: {:x}", cpu.reg_a);
        assert_eq!(0x03, cpu.reg_x, "Register X: {:x}", cpu.reg_x);
        assert_eq!(0x00, cpu.reg_y, "Register Y: {:x}", cpu.reg_y);
        assert_eq!(0xff, cpu.stack_pointer, "Stack Pointer: {:x}", cpu.stack_pointer);
        // assert_eq!(0x609, cpu.program_counter, "Program Counter: {:x}", cpu.program_counter);
        assert!(cpu.status.contains(
            CpuStatus::from_bits(0b00110011).unwrap()
        ), "CPU Status: {:b}", cpu.status.bits());

        assert_eq!(
            [cpu.bus.read(0x200), cpu.bus.read(0x201), cpu.bus.read(0x202)], 
            [0x03, 0x3, 0x00]
        );
    }
}