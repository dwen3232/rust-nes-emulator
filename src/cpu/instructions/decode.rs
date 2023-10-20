use super::{Opcode, AddressingMode, CpuCycleUnit};


pub fn decode_opcode(opcode: u8) -> Result<(Opcode, AddressingMode, CpuCycleUnit), String> {
    // Used this reference for decoding opcodes to Opcode addressing mode pairs
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
        0x69 => (Opcode::ADC, AddressingMode::Immediate, 2),
        0x65 => (Opcode::ADC, AddressingMode::ZeroPage, 3),
        0x75 => (Opcode::ADC, AddressingMode::ZeroPageIndexX, 4),
        0x6D => (Opcode::ADC, AddressingMode::Absolute, 4),
        0x7D => (Opcode::ADC, AddressingMode::AbsoluteIndexX, 4),
        0x79 => (Opcode::ADC, AddressingMode::AbsoluteIndexY, 4),
        0x61 => (Opcode::ADC, AddressingMode::IndirectX, 6),
        0x71 => (Opcode::ADC, AddressingMode::IndirectY, 5),
        // Immediate     AND #$44      $29  2   2
        // Zero Page     AND $44       $25  2   3
        // Zero Page,X   AND $44,X     $35  2   4
        // Absolute      AND $4400     $2D  3   4
        // Absolute,X    AND $4400,X   $3D  3   4+
        // Absolute,Y    AND $4400,Y   $39  3   4+
        // Indirect,X    AND ($44,X)   $21  2   6
        // Indirect,Y    AND ($44),Y   $31  2   5+
        0x29 => (Opcode::AND, AddressingMode::Immediate, 2),
        0x25 => (Opcode::AND, AddressingMode::ZeroPage, 3),
        0x35 => (Opcode::AND, AddressingMode::ZeroPageIndexX, 4),
        0x2D => (Opcode::AND, AddressingMode::Absolute, 4),
        0x3D => (Opcode::AND, AddressingMode::AbsoluteIndexX, 4),
        0x39 => (Opcode::AND, AddressingMode::AbsoluteIndexY, 4),
        0x21 => (Opcode::AND, AddressingMode::IndirectX, 6),
        0x31 => (Opcode::AND, AddressingMode::IndirectY, 5),
        // Accumulator   ASL A         $0A  1   2
        // Zero Page     ASL $44       $06  2   5
        // Zero Page,X   ASL $44,X     $16  2   6
        // Absolute      ASL $4400     $0E  3   6
        // Absolute,X    ASL $4400,X   $1E  3   7
        0x0A => (Opcode::ASL, AddressingMode::Accumulator, 2),
        0x06 => (Opcode::ASL, AddressingMode::ZeroPage, 5),
        0x16 => (Opcode::ASL, AddressingMode::ZeroPageIndexX, 6),
        0x0E => (Opcode::ASL, AddressingMode::Absolute, 6),
        0x1E => (Opcode::ASL, AddressingMode::AbsoluteIndexX, 7),
        // BPL (Branch on PLus)           $10
        // BMI (Branch on MInus)          $30
        // BVC (Branch on oVerflow Clear) $50
        // BVS (Branch on oVerflow Set)   $70
        // BCC (Branch on Carry Clear)    $90
        // BCS (Branch on Carry Set)      $B0
        // BNE (Branch on Not Equal)      $D0
        // BEQ (Branch on EQual)          $F0
        0x10 => (Opcode::BPL, AddressingMode::Relative, 2),
        0x30 => (Opcode::BMI, AddressingMode::Relative, 2),
        0x50 => (Opcode::BVC, AddressingMode::Relative, 2),
        0x70 => (Opcode::BVS, AddressingMode::Relative, 2),
        0x90 => (Opcode::BCC, AddressingMode::Relative, 2),
        0xB0 => (Opcode::BCS, AddressingMode::Relative, 2),
        0xD0 => (Opcode::BNE, AddressingMode::Relative, 2),
        0xF0 => (Opcode::BEQ, AddressingMode::Relative, 2),
        // Zero Page     BIT $44       $24  2   3
        // Absolute      BIT $4400     $2C  3   4
        0x24 => (Opcode::BIT, AddressingMode::ZeroPage, 3),
        0x2C => (Opcode::BIT, AddressingMode::Absolute, 4),
        // Implied       BRK           $00  1   7
        0x00 => (Opcode::BRK, AddressingMode::Implicit, 7),
        // Immediate     CMP #$44      $C9  2   2
        // Zero Page     CMP $44       $C5  2   3
        // Zero Page,X   CMP $44,X     $D5  2   4
        // Absolute      CMP $4400     $CD  3   4
        // Absolute,X    CMP $4400,X   $DD  3   4+
        // Absolute,Y    CMP $4400,Y   $D9  3   4+
        // Indirect,X    CMP ($44,X)   $C1  2   6
        // Indirect,Y    CMP ($44),Y   $D1  2   5+
        0xC9 => (Opcode::CMP, AddressingMode::Immediate, 2),
        0xC5 => (Opcode::CMP, AddressingMode::ZeroPage, 3),
        0xD5 => (Opcode::CMP, AddressingMode::ZeroPageIndexX, 4),
        0xCD => (Opcode::CMP, AddressingMode::Absolute, 4),
        0xDD => (Opcode::CMP, AddressingMode::AbsoluteIndexX, 4),
        0xD9 => (Opcode::CMP, AddressingMode::AbsoluteIndexY, 4),
        0xC1 => (Opcode::CMP, AddressingMode::IndirectX, 6),
        0xD1 => (Opcode::CMP, AddressingMode::IndirectY, 5),
        // Immediate     CPX #$44      $E0  2   2
        // Zero Page     CPX $44       $E4  2   3
        // Absolute      CPX $4400     $EC  3   4
        0xE0 => (Opcode::CPX, AddressingMode::Immediate, 2),
        0xE4 => (Opcode::CPX, AddressingMode::ZeroPage, 3),
        0xEC => (Opcode::CPX, AddressingMode::Absolute, 4),
        // Immediate     CPY #$44      $C0  2   2
        // Zero Page     CPY $44       $C4  2   3
        // Absolute      CPY $4400     $CC  3   4
        0xC0 => (Opcode::CPY, AddressingMode::Immediate, 2),
        0xC4 => (Opcode::CPY, AddressingMode::ZeroPage, 3),
        0xCC => (Opcode::CPY, AddressingMode::Absolute, 4),
        // Zero Page     DEC $44       $C6  2   5
        // Zero Page,X   DEC $44,X     $D6  2   6
        // Absolute      DEC $4400     $CE  3   6
        // Absolute,X    DEC $4400,X   $DE  3   7
        0xC6 => (Opcode::DEC, AddressingMode::ZeroPage, 5),
        0xD6 => (Opcode::DEC, AddressingMode::ZeroPageIndexX, 6),
        0xCE => (Opcode::DEC, AddressingMode::Absolute, 6),
        0xDE => (Opcode::DEC, AddressingMode::AbsoluteIndexX, 7),
        // Immediate     EOR #$44      $49  2   2
        // Zero Page     EOR $44       $45  2   3
        // Zero Page,X   EOR $44,X     $55  2   4
        // Absolute      EOR $4400     $4D  3   4
        // Absolute,X    EOR $4400,X   $5D  3   4+
        // Absolute,Y    EOR $4400,Y   $59  3   4+
        // Indirect,X    EOR ($44,X)   $41  2   6
        // Indirect,Y    EOR ($44),Y   $51  2   5+
        0x49 => (Opcode::EOR, AddressingMode::Immediate, 2),
        0x45 => (Opcode::EOR, AddressingMode::ZeroPage, 3),
        0x55 => (Opcode::EOR, AddressingMode::ZeroPageIndexX, 4),
        0x4D => (Opcode::EOR, AddressingMode::Absolute, 4),
        0x5D => (Opcode::EOR, AddressingMode::AbsoluteIndexX, 4),
        0x59 => (Opcode::EOR, AddressingMode::AbsoluteIndexY, 4),
        0x41 => (Opcode::EOR, AddressingMode::IndirectX, 6),
        0x51 => (Opcode::EOR, AddressingMode::IndirectY, 5),
        // CLC (CLear Carry)              $18
        // SEC (SEt Carry)                $38
        // CLI (CLear Interrupt)          $58
        // SEI (SEt Interrupt)            $78
        // CLV (CLear oVerflow)           $B8
        // CLD (CLear Decimal)            $D8
        // SED (SEt Decimal)              $F8
        0x18 => (Opcode::CLC, AddressingMode::Implicit, 2),
        0x38 => (Opcode::SEC, AddressingMode::Implicit, 2),
        0x58 => (Opcode::CLI, AddressingMode::Implicit, 2),
        0x78 => (Opcode::SEI, AddressingMode::Implicit, 2),
        0xB8 => (Opcode::CLV, AddressingMode::Implicit, 2),
        0xD8 => (Opcode::CLD, AddressingMode::Implicit, 2),
        0xF8 => (Opcode::SED, AddressingMode::Implicit, 2),
        // Zero Page     INC $44       $E6  2   5
        // Zero Page,X   INC $44,X     $F6  2   6
        // Absolute      INC $4400     $EE  3   6
        // Absolute,X    INC $4400,X   $FE  3   7
        0xE6 => (Opcode::INC, AddressingMode::ZeroPage, 5),
        0xF6 => (Opcode::INC, AddressingMode::ZeroPageIndexX, 6),
        0xEE => (Opcode::INC, AddressingMode::Absolute, 6),
        0xFE => (Opcode::INC, AddressingMode::AbsoluteIndexX, 7),
        // Absolute      JMP $5597     $4C  3   3
        // Indirect      JMP ($5597)   $6C  3   5
        0x4C => (Opcode::JMP, AddressingMode::AbsoluteJump, 3),
        0x6C => (Opcode::JMP, AddressingMode::IndirectJump, 5),
        // Absolute      JSR $5597     $20  3   6
        0x20 => (Opcode::JSR, AddressingMode::AbsoluteJump, 6),
        // Immediate     LDA #$44      $A9  2   2
        // Zero Page     LDA $44       $A5  2   3
        // Zero Page,X   LDA $44,X     $B5  2   4
        // Absolute      LDA $4400     $AD  3   4
        // Absolute,X    LDA $4400,X   $BD  3   4+
        // Absolute,Y    LDA $4400,Y   $B9  3   4+
        // Indirect,X    LDA ($44,X)   $A1  2   6
        // Indirect,Y    LDA ($44),Y   $B1  2   5+
        0xA9 => (Opcode::LDA, AddressingMode::Immediate, 2),
        0xA5 => (Opcode::LDA, AddressingMode::ZeroPage, 3),
        0xB5 => (Opcode::LDA, AddressingMode::ZeroPageIndexX, 4),
        0xAD => (Opcode::LDA, AddressingMode::Absolute, 4),
        0xBD => (Opcode::LDA, AddressingMode::AbsoluteIndexX, 4),
        0xB9 => (Opcode::LDA, AddressingMode::AbsoluteIndexY, 4),
        0xA1 => (Opcode::LDA, AddressingMode::IndirectX, 6),
        0xB1 => (Opcode::LDA, AddressingMode::IndirectY, 5),
        // Immediate     LDX #$44      $A2  2   2
        // Zero Page     LDX $44       $A6  2   3
        // Zero Page,Y   LDX $44,Y     $B6  2   4
        // Absolute      LDX $4400     $AE  3   4
        // Absolute,Y    LDX $4400,Y   $BE  3   4+
        0xA2 => (Opcode::LDX, AddressingMode::Immediate, 2),
        0xA6 => (Opcode::LDX, AddressingMode::ZeroPage, 3),
        0xB6 => (Opcode::LDX, AddressingMode::ZeroPageIndexY, 4),
        0xAE => (Opcode::LDX, AddressingMode::Absolute, 4),
        0xBE => (Opcode::LDX, AddressingMode::AbsoluteIndexY, 4),
        // Immediate     LDY #$44      $A0  2   2
        // Zero Page     LDY $44       $A4  2   3
        // Zero Page,X   LDY $44,X     $B4  2   4
        // Absolute      LDY $4400     $AC  3   4
        // Absolute,X    LDY $4400,X   $BC  3   4+
        0xA0 => (Opcode::LDY, AddressingMode::Immediate, 2),
        0xA4 => (Opcode::LDY, AddressingMode::ZeroPage, 3),
        0xB4 => (Opcode::LDY, AddressingMode::ZeroPageIndexX, 4),
        0xAC => (Opcode::LDY, AddressingMode::Absolute, 4),
        0xBC => (Opcode::LDY, AddressingMode::AbsoluteIndexX, 4),
        // Accumulator   LSR A         $4A  1   2
        // Zero Page     LSR $44       $46  2   5
        // Zero Page,X   LSR $44,X     $56  2   6
        // Absolute      LSR $4400     $4E  3   6
        // Absolute,X    LSR $4400,X   $5E  3   7
        0x4A => (Opcode::LSR, AddressingMode::Accumulator, 2),
        0x46 => (Opcode::LSR, AddressingMode::ZeroPage, 5),
        0x56 => (Opcode::LSR, AddressingMode::ZeroPageIndexX, 6),
        0x4E => (Opcode::LSR, AddressingMode::Absolute, 6),
        0x5E => (Opcode::LSR, AddressingMode::AbsoluteIndexX, 7),
        // Implied       NOP           $EA  1   2
        0xEA => (Opcode::NOP, AddressingMode::Implicit, 2),
        // Immediate     ORA #$44      $09  2   2
        // Zero Page     ORA $44       $05  2   3
        // Zero Page,X   ORA $44,X     $15  2   4
        // Absolute      ORA $4400     $0D  3   4
        // Absolute,X    ORA $4400,X   $1D  3   4+
        // Absolute,Y    ORA $4400,Y   $19  3   4+
        // Indirect,X    ORA ($44,X)   $01  2   6
        // Indirect,Y    ORA ($44),Y   $11  2   5+
        0x09 => (Opcode::ORA, AddressingMode::Immediate, 2),
        0x05 => (Opcode::ORA, AddressingMode::ZeroPage, 3),
        0x15 => (Opcode::ORA, AddressingMode::ZeroPageIndexX, 4),
        0x0D => (Opcode::ORA, AddressingMode::Absolute, 4),
        0x1D => (Opcode::ORA, AddressingMode::AbsoluteIndexX, 4),
        0x19 => (Opcode::ORA, AddressingMode::AbsoluteIndexY, 4),
        0x01 => (Opcode::ORA, AddressingMode::IndirectX, 6),
        0x11 => (Opcode::ORA, AddressingMode::IndirectY, 5),
        // TAX (Transfer A to X)    $AA
        // TXA (Transfer X to A)    $8A
        // DEX (DEcrement X)        $CA
        // INX (INcrement X)        $E8
        // TAY (Transfer A to Y)    $A8
        // TYA (Transfer Y to A)    $98
        // DEY (DEcrement Y)        $88
        // INY (INcrement Y)        $C8
        0xAA => (Opcode::TAX, AddressingMode::Implicit, 2),
        0x8A => (Opcode::TXA, AddressingMode::Implicit, 2),
        0xCA => (Opcode::DEX, AddressingMode::Implicit, 2),
        0xE8 => (Opcode::INX, AddressingMode::Implicit, 2),
        0xA8 => (Opcode::TAY, AddressingMode::Implicit, 2),
        0x98 => (Opcode::TYA, AddressingMode::Implicit, 2),
        0x88 => (Opcode::DEY, AddressingMode::Implicit, 2),
        0xC8 => (Opcode::INY, AddressingMode::Implicit, 2),
        // Accumulator   ROL A         $2A  1   2
        // Zero Page     ROL $44       $26  2   5
        // Zero Page,X   ROL $44,X     $36  2   6
        // Absolute      ROL $4400     $2E  3   6
        // Absolute,X    ROL $4400,X   $3E  3   7
        0x2A => (Opcode::ROL, AddressingMode::Accumulator, 2),
        0x26 => (Opcode::ROL, AddressingMode::ZeroPage, 5),
        0x36 => (Opcode::ROL, AddressingMode::ZeroPageIndexX, 6),
        0x2E => (Opcode::ROL, AddressingMode::Absolute, 6),
        0x3E => (Opcode::ROL, AddressingMode::AbsoluteIndexX, 7),
        // Accumulator   ROR A         $6A  1   2
        // Zero Page     ROR $44       $66  2   5
        // Zero Page,X   ROR $44,X     $76  2   6
        // Absolute      ROR $4400     $6E  3   6
        // Absolute,X    ROR $4400,X   $7E  3   7
        0x6A => (Opcode::ROR, AddressingMode::Accumulator, 2),
        0x66 => (Opcode::ROR, AddressingMode::ZeroPage, 5),
        0x76 => (Opcode::ROR, AddressingMode::ZeroPageIndexX, 6),
        0x6E => (Opcode::ROR, AddressingMode::Absolute, 6),
        0x7E => (Opcode::ROR, AddressingMode::AbsoluteIndexX, 7),
        // Implied       RTI           $40  1   6
        0x40 => (Opcode::RTI, AddressingMode::Implicit, 6),
        // Implied       RTS           $60  1   6
        0x60 => (Opcode::RTS, AddressingMode::Implicit, 6),
        // Immediate     SBC #$44      $E9  2   2
        // Zero Page     SBC $44       $E5  2   3
        // Zero Page,X   SBC $44,X     $F5  2   4
        // Absolute      SBC $4400     $ED  3   4
        // Absolute,X    SBC $4400,X   $FD  3   4+
        // Absolute,Y    SBC $4400,Y   $F9  3   4+
        // Indirect,X    SBC ($44,X)   $E1  2   6
        // Indirect,Y    SBC ($44),Y   $F1  2   5+
        0xE9 => (Opcode::SBC, AddressingMode::Immediate, 2),
        0xE5 => (Opcode::SBC, AddressingMode::ZeroPage, 3),
        0xF5 => (Opcode::SBC, AddressingMode::ZeroPageIndexX, 4),
        0xED => (Opcode::SBC, AddressingMode::Absolute, 4),
        0xFD => (Opcode::SBC, AddressingMode::AbsoluteIndexX, 4),
        0xF9 => (Opcode::SBC, AddressingMode::AbsoluteIndexY, 4),
        0xE1 => (Opcode::SBC, AddressingMode::IndirectX, 6),
        0xF1 => (Opcode::SBC, AddressingMode::IndirectY, 5),
        // Zero Page     STA $44       $85  2   3
        // Zero Page,X   STA $44,X     $95  2   4
        // Absolute      STA $4400     $8D  3   4
        // Absolute,X    STA $4400,X   $9D  3   5
        // Absolute,Y    STA $4400,Y   $99  3   5
        // Indirect,X    STA ($44,X)   $81  2   6
        // Indirect,Y    STA ($44),Y   $91  2   6
        0x85 => (Opcode::STA, AddressingMode::ZeroPage, 3),
        0x95 => (Opcode::STA, AddressingMode::ZeroPageIndexX, 4),
        0x8D => (Opcode::STA, AddressingMode::Absolute, 4),
        0x9D => (Opcode::STA, AddressingMode::AbsoluteIndexX, 5),
        0x99 => (Opcode::STA, AddressingMode::AbsoluteIndexY, 5),
        0x81 => (Opcode::STA, AddressingMode::IndirectX, 6),
        0x91 => (Opcode::STA, AddressingMode::IndirectY, 6),
        // TXS (Transfer X to Stack ptr)   $9A  2
        // TSX (Transfer Stack ptr to X)   $BA  2
        // PHA (PusH Accumulator)          $48  3
        // PLA (PuLl Accumulator)          $68  4
        // PHP (PusH Processor status)     $08  3
        // PLP (PuLl Processor status)     $28  4
        0x9A => (Opcode::TXS, AddressingMode::Implicit, 2),
        0xBA => (Opcode::TSX, AddressingMode::Implicit, 2),
        0x48 => (Opcode::PHA, AddressingMode::Implicit, 3),
        0x68 => (Opcode::PLA, AddressingMode::Implicit, 4),
        0x08 => (Opcode::PHP, AddressingMode::Implicit, 3),
        0x28 => (Opcode::PLP, AddressingMode::Implicit, 4),
        // Zero Page     STX $44       $86  2   3
        // Zero Page,Y   STX $44,Y     $96  2   4
        // Absolute      STX $4400     $8E  3   4
        0x86 => (Opcode::STX, AddressingMode::ZeroPage, 3),
        0x96 => (Opcode::STX, AddressingMode::ZeroPageIndexY, 4),
        0x8E => (Opcode::STX, AddressingMode::Absolute, 4),
        // Zero Page     STY $44       $84  2   3
        // Zero Page,X   STY $44,X     $94  2   4
        // Absolute      STY $4400     $8C  3   4
        0x84 => (Opcode::STY, AddressingMode::ZeroPage, 3),
        0x94 => (Opcode::STY, AddressingMode::ZeroPageIndexX, 4),
        0x8C => (Opcode::STY, AddressingMode::Absolute, 4),
        _ => {
            return Err(format!("Opcode not implemented {:2x}", opcode))
        }
    };
    Ok(result)
}