#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
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


pub fn decode_opcode(opcode: u8) -> Result<(Instruction, AddressingMode, u8), String> {
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
        0x69 => (Instruction::ADC, AddressingMode::Immediate, 2),
        0x65 => (Instruction::ADC, AddressingMode::ZeroPage, 3),
        0x75 => (Instruction::ADC, AddressingMode::ZeroPageIndexX, 4),
        0x6D => (Instruction::ADC, AddressingMode::Absolute, 4),
        0x7D => (Instruction::ADC, AddressingMode::AbsoluteIndexX, 4),
        0x79 => (Instruction::ADC, AddressingMode::AbsoluteIndexY, 4),
        0x61 => (Instruction::ADC, AddressingMode::IndirectX, 6),
        0x71 => (Instruction::ADC, AddressingMode::IndirectY, 5),
        // Immediate     AND #$44      $29  2   2
        // Zero Page     AND $44       $25  2   3
        // Zero Page,X   AND $44,X     $35  2   4
        // Absolute      AND $4400     $2D  3   4
        // Absolute,X    AND $4400,X   $3D  3   4+
        // Absolute,Y    AND $4400,Y   $39  3   4+
        // Indirect,X    AND ($44,X)   $21  2   6
        // Indirect,Y    AND ($44),Y   $31  2   5+
        0x29 => (Instruction::AND, AddressingMode::Immediate, 2),
        0x25 => (Instruction::AND, AddressingMode::ZeroPage, 3),
        0x35 => (Instruction::AND, AddressingMode::ZeroPageIndexX, 4),
        0x2D => (Instruction::AND, AddressingMode::Absolute, 4),
        0x3D => (Instruction::AND, AddressingMode::AbsoluteIndexX, 4),
        0x39 => (Instruction::AND, AddressingMode::AbsoluteIndexY, 4),
        0x21 => (Instruction::AND, AddressingMode::IndirectX, 6),
        0x31 => (Instruction::AND, AddressingMode::IndirectY, 5),
        // Accumulator   ASL A         $0A  1   2
        // Zero Page     ASL $44       $06  2   5
        // Zero Page,X   ASL $44,X     $16  2   6
        // Absolute      ASL $4400     $0E  3   6
        // Absolute,X    ASL $4400,X   $1E  3   7
        0x0A => (Instruction::ASL, AddressingMode::Accumulator, 2),
        0x06 => (Instruction::ASL, AddressingMode::ZeroPage, 5),
        0x16 => (Instruction::ASL, AddressingMode::ZeroPageIndexX, 6),
        0x0E => (Instruction::ASL, AddressingMode::Absolute, 6),
        0x1E => (Instruction::ASL, AddressingMode::AbsoluteIndexX, 7),
        // BPL (Branch on PLus)           $10
        // BMI (Branch on MInus)          $30
        // BVC (Branch on oVerflow Clear) $50
        // BVS (Branch on oVerflow Set)   $70
        // BCC (Branch on Carry Clear)    $90
        // BCS (Branch on Carry Set)      $B0
        // BNE (Branch on Not Equal)      $D0
        // BEQ (Branch on EQual)          $F0
        0x10 => (Instruction::BPL, AddressingMode::Relative, 2),
        0x30 => (Instruction::BMI, AddressingMode::Relative, 2),
        0x50 => (Instruction::BVC, AddressingMode::Relative, 2),
        0x70 => (Instruction::BVS, AddressingMode::Relative, 2),
        0x90 => (Instruction::BCC, AddressingMode::Relative, 2),
        0xB0 => (Instruction::BCS, AddressingMode::Relative, 2),
        0xD0 => (Instruction::BNE, AddressingMode::Relative, 2),
        0xF0 => (Instruction::BEQ, AddressingMode::Relative, 2),
        // Zero Page     BIT $44       $24  2   3
        // Absolute      BIT $4400     $2C  3   4
        0x24 => (Instruction::BIT, AddressingMode::ZeroPage, 3),
        0x2C => (Instruction::BIT, AddressingMode::Absolute, 4),
        // Implied       BRK           $00  1   7
        0x00 => (Instruction::BRK, AddressingMode::Implicit, 7),
        // Immediate     CMP #$44      $C9  2   2
        // Zero Page     CMP $44       $C5  2   3
        // Zero Page,X   CMP $44,X     $D5  2   4
        // Absolute      CMP $4400     $CD  3   4
        // Absolute,X    CMP $4400,X   $DD  3   4+
        // Absolute,Y    CMP $4400,Y   $D9  3   4+
        // Indirect,X    CMP ($44,X)   $C1  2   6
        // Indirect,Y    CMP ($44),Y   $D1  2   5+
        0xC9 => (Instruction::CMP, AddressingMode::Immediate, 2),
        0xC5 => (Instruction::CMP, AddressingMode::ZeroPage, 3),
        0xD5 => (Instruction::CMP, AddressingMode::ZeroPageIndexX, 4),
        0xCD => (Instruction::CMP, AddressingMode::Absolute, 4),
        0xDD => (Instruction::CMP, AddressingMode::AbsoluteIndexX, 4),
        0xD9 => (Instruction::CMP, AddressingMode::AbsoluteIndexY, 4),
        0xC1 => (Instruction::CMP, AddressingMode::IndirectX, 6),
        0xD1 => (Instruction::CMP, AddressingMode::IndirectY, 5),
        // Immediate     CPX #$44      $E0  2   2
        // Zero Page     CPX $44       $E4  2   3
        // Absolute      CPX $4400     $EC  3   4
        0xE0 => (Instruction::CPX, AddressingMode::Immediate, 2),
        0xE4 => (Instruction::CPX, AddressingMode::ZeroPage, 3),
        0xEC => (Instruction::CPX, AddressingMode::Absolute, 4),
        // Immediate     CPY #$44      $C0  2   2
        // Zero Page     CPY $44       $C4  2   3
        // Absolute      CPY $4400     $CC  3   4
        0xC0 => (Instruction::CPY, AddressingMode::Immediate, 2),
        0xC4 => (Instruction::CPY, AddressingMode::ZeroPage, 3),
        0xCC => (Instruction::CPY, AddressingMode::Absolute, 4),
        // Zero Page     DEC $44       $C6  2   5
        // Zero Page,X   DEC $44,X     $D6  2   6
        // Absolute      DEC $4400     $CE  3   6
        // Absolute,X    DEC $4400,X   $DE  3   7
        0xC6 => (Instruction::DEC, AddressingMode::ZeroPage, 5),
        0xD6 => (Instruction::DEC, AddressingMode::ZeroPageIndexX, 6),
        0xCE => (Instruction::DEC, AddressingMode::Absolute, 6),
        0xDE => (Instruction::DEC, AddressingMode::AbsoluteIndexX, 7),
        // Immediate     EOR #$44      $49  2   2
        // Zero Page     EOR $44       $45  2   3
        // Zero Page,X   EOR $44,X     $55  2   4
        // Absolute      EOR $4400     $4D  3   4
        // Absolute,X    EOR $4400,X   $5D  3   4+
        // Absolute,Y    EOR $4400,Y   $59  3   4+
        // Indirect,X    EOR ($44,X)   $41  2   6
        // Indirect,Y    EOR ($44),Y   $51  2   5+
        0x49 => (Instruction::EOR, AddressingMode::Immediate, 2),
        0x45 => (Instruction::EOR, AddressingMode::ZeroPage, 3),
        0x55 => (Instruction::EOR, AddressingMode::ZeroPageIndexX, 4),
        0x4D => (Instruction::EOR, AddressingMode::Absolute, 4),
        0x5D => (Instruction::EOR, AddressingMode::AbsoluteIndexX, 4),
        0x59 => (Instruction::EOR, AddressingMode::AbsoluteIndexY, 4),
        0x41 => (Instruction::EOR, AddressingMode::IndirectX, 6),
        0x51 => (Instruction::EOR, AddressingMode::IndirectY, 5),
        // CLC (CLear Carry)              $18
        // SEC (SEt Carry)                $38
        // CLI (CLear Interrupt)          $58
        // SEI (SEt Interrupt)            $78
        // CLV (CLear oVerflow)           $B8
        // CLD (CLear Decimal)            $D8
        // SED (SEt Decimal)              $F8
        0x18 => (Instruction::CLC, AddressingMode::Implicit, 2),
        0x38 => (Instruction::SEC, AddressingMode::Implicit, 2),
        0x58 => (Instruction::CLI, AddressingMode::Implicit, 2),
        0x78 => (Instruction::SEI, AddressingMode::Implicit, 2),
        0xB8 => (Instruction::CLV, AddressingMode::Implicit, 2),
        0xD8 => (Instruction::CLD, AddressingMode::Implicit, 2),
        0xF8 => (Instruction::SED, AddressingMode::Implicit, 2),
        // Zero Page     INC $44       $E6  2   5
        // Zero Page,X   INC $44,X     $F6  2   6
        // Absolute      INC $4400     $EE  3   6
        // Absolute,X    INC $4400,X   $FE  3   7
        0xE6 => (Instruction::INC, AddressingMode::ZeroPage, 5),
        0xF6 => (Instruction::INC, AddressingMode::ZeroPageIndexX, 6),
        0xEE => (Instruction::INC, AddressingMode::Absolute, 6),
        0xFE => (Instruction::INC, AddressingMode::AbsoluteIndexX, 7),
        // Absolute      JMP $5597     $4C  3   3
        // Indirect      JMP ($5597)   $6C  3   5
        0x4C => (Instruction::JMP, AddressingMode::AbsoluteJump, 3),
        0x6C => (Instruction::JMP, AddressingMode::IndirectJump, 5),
        // Absolute      JSR $5597     $20  3   6
        0x20 => (Instruction::JSR, AddressingMode::AbsoluteJump, 6),
        // Immediate     LDA #$44      $A9  2   2
        // Zero Page     LDA $44       $A5  2   3
        // Zero Page,X   LDA $44,X     $B5  2   4
        // Absolute      LDA $4400     $AD  3   4
        // Absolute,X    LDA $4400,X   $BD  3   4+
        // Absolute,Y    LDA $4400,Y   $B9  3   4+
        // Indirect,X    LDA ($44,X)   $A1  2   6
        // Indirect,Y    LDA ($44),Y   $B1  2   5+
        0xA9 => (Instruction::LDA, AddressingMode::Immediate, 2),
        0xA5 => (Instruction::LDA, AddressingMode::ZeroPage, 3),
        0xB5 => (Instruction::LDA, AddressingMode::ZeroPageIndexX, 4),
        0xAD => (Instruction::LDA, AddressingMode::Absolute, 4),
        0xBD => (Instruction::LDA, AddressingMode::AbsoluteIndexX, 4),
        0xB9 => (Instruction::LDA, AddressingMode::AbsoluteIndexY, 4),
        0xA1 => (Instruction::LDA, AddressingMode::IndirectX, 6),
        0xB1 => (Instruction::LDA, AddressingMode::IndirectY, 5),
        // Immediate     LDX #$44      $A2  2   2
        // Zero Page     LDX $44       $A6  2   3
        // Zero Page,Y   LDX $44,Y     $B6  2   4
        // Absolute      LDX $4400     $AE  3   4
        // Absolute,Y    LDX $4400,Y   $BE  3   4+
        0xA2 => (Instruction::LDX, AddressingMode::Immediate, 2),
        0xA6 => (Instruction::LDX, AddressingMode::ZeroPage, 3),
        0xB6 => (Instruction::LDX, AddressingMode::ZeroPageIndexY, 4),
        0xAE => (Instruction::LDX, AddressingMode::Absolute, 4),
        0xBE => (Instruction::LDX, AddressingMode::AbsoluteIndexY, 4),
        // Immediate     LDY #$44      $A0  2   2
        // Zero Page     LDY $44       $A4  2   3
        // Zero Page,X   LDY $44,X     $B4  2   4
        // Absolute      LDY $4400     $AC  3   4
        // Absolute,X    LDY $4400,X   $BC  3   4+
        0xA0 => (Instruction::LDY, AddressingMode::Immediate, 2),
        0xA4 => (Instruction::LDY, AddressingMode::ZeroPage, 3),
        0xB4 => (Instruction::LDY, AddressingMode::ZeroPageIndexX, 4),
        0xAC => (Instruction::LDY, AddressingMode::Absolute, 4),
        0xBC => (Instruction::LDY, AddressingMode::AbsoluteIndexX, 4),
        // Accumulator   LSR A         $4A  1   2
        // Zero Page     LSR $44       $46  2   5
        // Zero Page,X   LSR $44,X     $56  2   6
        // Absolute      LSR $4400     $4E  3   6
        // Absolute,X    LSR $4400,X   $5E  3   7
        0x4A => (Instruction::LSR, AddressingMode::Accumulator, 2),
        0x46 => (Instruction::LSR, AddressingMode::ZeroPage, 5),
        0x56 => (Instruction::LSR, AddressingMode::ZeroPageIndexX, 6),
        0x4E => (Instruction::LSR, AddressingMode::Absolute, 6),
        0x5E => (Instruction::LSR, AddressingMode::AbsoluteIndexX, 7),
        // Implied       NOP           $EA  1   2
        0xEA => (Instruction::NOP, AddressingMode::Implicit, 2),
        // Immediate     ORA #$44      $09  2   2
        // Zero Page     ORA $44       $05  2   3
        // Zero Page,X   ORA $44,X     $15  2   4
        // Absolute      ORA $4400     $0D  3   4
        // Absolute,X    ORA $4400,X   $1D  3   4+
        // Absolute,Y    ORA $4400,Y   $19  3   4+
        // Indirect,X    ORA ($44,X)   $01  2   6
        // Indirect,Y    ORA ($44),Y   $11  2   5+
        0x09 => (Instruction::ORA, AddressingMode::Immediate, 2),
        0x05 => (Instruction::ORA, AddressingMode::ZeroPage, 3),
        0x15 => (Instruction::ORA, AddressingMode::ZeroPageIndexX, 4),
        0x0D => (Instruction::ORA, AddressingMode::Absolute, 4),
        0x1D => (Instruction::ORA, AddressingMode::AbsoluteIndexX, 4),
        0x19 => (Instruction::ORA, AddressingMode::AbsoluteIndexY, 4),
        0x01 => (Instruction::ORA, AddressingMode::IndirectX, 6),
        0x11 => (Instruction::ORA, AddressingMode::IndirectY, 5),
        // TAX (Transfer A to X)    $AA
        // TXA (Transfer X to A)    $8A
        // DEX (DEcrement X)        $CA
        // INX (INcrement X)        $E8
        // TAY (Transfer A to Y)    $A8
        // TYA (Transfer Y to A)    $98
        // DEY (DEcrement Y)        $88
        // INY (INcrement Y)        $C8
        0xAA => (Instruction::TAX, AddressingMode::Implicit, 2),
        0x8A => (Instruction::TXA, AddressingMode::Implicit, 2),
        0xCA => (Instruction::DEX, AddressingMode::Implicit, 2),
        0xE8 => (Instruction::INX, AddressingMode::Implicit, 2),
        0xA8 => (Instruction::TAY, AddressingMode::Implicit, 2),
        0x98 => (Instruction::TYA, AddressingMode::Implicit, 2),
        0x88 => (Instruction::DEY, AddressingMode::Implicit, 2),
        0xC8 => (Instruction::INY, AddressingMode::Implicit, 2),
        // Accumulator   ROL A         $2A  1   2
        // Zero Page     ROL $44       $26  2   5
        // Zero Page,X   ROL $44,X     $36  2   6
        // Absolute      ROL $4400     $2E  3   6
        // Absolute,X    ROL $4400,X   $3E  3   7
        0x2A => (Instruction::ROL, AddressingMode::Accumulator, 2),
        0x26 => (Instruction::ROL, AddressingMode::ZeroPage, 5),
        0x36 => (Instruction::ROL, AddressingMode::ZeroPageIndexX, 6),
        0x2E => (Instruction::ROL, AddressingMode::Absolute, 6),
        0x3E => (Instruction::ROL, AddressingMode::AbsoluteIndexX, 7),
        // Accumulator   ROR A         $6A  1   2
        // Zero Page     ROR $44       $66  2   5
        // Zero Page,X   ROR $44,X     $76  2   6
        // Absolute      ROR $4400     $6E  3   6
        // Absolute,X    ROR $4400,X   $7E  3   7
        0x6A => (Instruction::ROR, AddressingMode::Accumulator, 2),
        0x66 => (Instruction::ROR, AddressingMode::ZeroPage, 5),
        0x76 => (Instruction::ROR, AddressingMode::ZeroPageIndexX, 6),
        0x6E => (Instruction::ROR, AddressingMode::Absolute, 6),
        0x7E => (Instruction::ROR, AddressingMode::AbsoluteIndexX, 7),
        // Implied       RTI           $40  1   6
        0x40 => (Instruction::RTI, AddressingMode::Implicit, 6),
        // Implied       RTS           $60  1   6
        0x60 => (Instruction::RTS, AddressingMode::Implicit, 6),
        // Immediate     SBC #$44      $E9  2   2
        // Zero Page     SBC $44       $E5  2   3
        // Zero Page,X   SBC $44,X     $F5  2   4
        // Absolute      SBC $4400     $ED  3   4
        // Absolute,X    SBC $4400,X   $FD  3   4+
        // Absolute,Y    SBC $4400,Y   $F9  3   4+
        // Indirect,X    SBC ($44,X)   $E1  2   6
        // Indirect,Y    SBC ($44),Y   $F1  2   5+
        0xE9 => (Instruction::SBC, AddressingMode::Immediate, 2),
        0xE5 => (Instruction::SBC, AddressingMode::ZeroPage, 3),
        0xF5 => (Instruction::SBC, AddressingMode::ZeroPageIndexX, 4),
        0xED => (Instruction::SBC, AddressingMode::Absolute, 4),
        0xFD => (Instruction::SBC, AddressingMode::AbsoluteIndexX, 4),
        0xF9 => (Instruction::SBC, AddressingMode::AbsoluteIndexY, 4),
        0xE1 => (Instruction::SBC, AddressingMode::IndirectX, 6),
        0xF1 => (Instruction::SBC, AddressingMode::IndirectY, 5),
        // Zero Page     STA $44       $85  2   3
        // Zero Page,X   STA $44,X     $95  2   4
        // Absolute      STA $4400     $8D  3   4
        // Absolute,X    STA $4400,X   $9D  3   5
        // Absolute,Y    STA $4400,Y   $99  3   5
        // Indirect,X    STA ($44,X)   $81  2   6
        // Indirect,Y    STA ($44),Y   $91  2   6
        0x85 => (Instruction::STA, AddressingMode::ZeroPage, 3),
        0x95 => (Instruction::STA, AddressingMode::ZeroPageIndexX, 4),
        0x8D => (Instruction::STA, AddressingMode::Absolute, 4),
        0x9D => (Instruction::STA, AddressingMode::AbsoluteIndexX, 5),
        0x99 => (Instruction::STA, AddressingMode::AbsoluteIndexY, 5),
        0x81 => (Instruction::STA, AddressingMode::IndirectX, 6),
        0x91 => (Instruction::STA, AddressingMode::IndirectY, 6),
        // TXS (Transfer X to Stack ptr)   $9A  2
        // TSX (Transfer Stack ptr to X)   $BA  2
        // PHA (PusH Accumulator)          $48  3
        // PLA (PuLl Accumulator)          $68  4
        // PHP (PusH Processor status)     $08  3
        // PLP (PuLl Processor status)     $28  4
        0x9A => (Instruction::TXS, AddressingMode::Implicit, 2),
        0xBA => (Instruction::TSX, AddressingMode::Implicit, 2),
        0x48 => (Instruction::PHA, AddressingMode::Implicit, 3),
        0x68 => (Instruction::PLA, AddressingMode::Implicit, 4),
        0x08 => (Instruction::PHP, AddressingMode::Implicit, 3),
        0x28 => (Instruction::PLP, AddressingMode::Implicit, 4),
        // Zero Page     STX $44       $86  2   3
        // Zero Page,Y   STX $44,Y     $96  2   4
        // Absolute      STX $4400     $8E  3   4
        0x86 => (Instruction::STX, AddressingMode::ZeroPage, 3),
        0x96 => (Instruction::STX, AddressingMode::ZeroPageIndexY, 4),
        0x8E => (Instruction::STX, AddressingMode::Absolute, 4),
        // Zero Page     STY $44       $84  2   3
        // Zero Page,X   STY $44,X     $94  2   4
        // Absolute      STY $4400     $8C  3   4
        0x84 => (Instruction::STY, AddressingMode::ZeroPage, 3),
        0x94 => (Instruction::STY, AddressingMode::ZeroPageIndexX, 4),
        0x8C => (Instruction::STY, AddressingMode::Absolute, 4),
        _ => {
            return Err(format!("Opcode not implemented {:2x}", opcode))
        }
    };
    Ok(result)
}