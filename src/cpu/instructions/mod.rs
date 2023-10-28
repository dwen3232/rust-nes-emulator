mod decode;
mod parse;
mod execute;

use decode::decode_opcode;

pub use execute::execute_instruction;
pub use parse::parse_instruction;

type CpuCycleUnit = u8;

#[derive(Debug, Clone, Copy)]
pub struct Instruction {
    pub opcode: Opcode,
    pub param: Param,
    pub cycles: CpuCycleUnit,
}

// TODO! This is a misuse of Enums, make Opcode an Enum with no value and change the current implementation to a struct
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Opcode { // Reorder these at some point to something more logical
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
    // Stack instructions
    TXS, 
    TSX, 
    PHA, 
    PLA, 
    PHP, 
    PLP,
    STA, 
    STX, 
    STY,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Param {    // used by an instruction
    Value(u8),
    Address(u16),
    None
}

#[derive(Debug, PartialEq, Clone, Copy)]
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

