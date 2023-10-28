use crate::controller::{ControllerState, Controller};
use crate::cpu::{
    CpuAction, CpuState, CPU, Instruction
};
// use crate::ppu::ppu_state::PpuState;
use crate::ppu::{
    PpuAction, PpuState, PPU
};
use crate::rom::ROM;


// TODO: replace these!
type ProgramTrace = Vec<String>;

pub trait NES {
    // pub fn next_cpu_cycle();
    
    // Updates state to after next CPU instruction
    fn next_cpu_instruction(&mut self) -> Result<Instruction, String>;

    // Updates state to after next PPU cycle (next frame)
    fn next_ppu_cycle(&mut self) -> Result<(), String>;
    
    // Loads a program
    fn load_rom(&mut self, rom: ROM) -> Result<(), String>;

    // Resets the console
    fn reset(&mut self) -> Result<(), String>;

    // Look into CPU state
    fn peek_cpu_state(&self) -> CpuState;

    // Look into PPU state
    fn peek_ppu_state(&self) -> PpuState;

    fn create_program_trace(&self) -> Result<ProgramTrace, String>;
}

#[derive(Debug, Clone, Copy)]
struct ActionNES {
    cpu_state: CpuState,
    ppu_state: PpuState,
    con_state: Controller,
    rom_state: ROM,

    // program_loader:
}

impl ActionNES {
    fn trace_instruction(nes: &ActionNES, instruction: &Instruction, length: u16) -> String {
        let prev_counter = nes.cpu_state.program_counter;

        // decode instruction and addressing mode
        let opcode = cpu.read_byte_from_pc();
        let (instruction, addressing_mode, _) = decode_opcode(opcode)?;

        // parse instruction parameter 
        let param = cpu.read_arg(&addressing_mode);
        let length = cpu.program_counter - prev_counter;

        cpu.program_counter = prev_counter;     // revert program_counter

        let mut hex_dump = vec![];
        // add opcode byte to dump
        hex_dump.push(opcode);


        // get the parsed arg as a u16
        let arg = match length {
            1 => 0,
            2 => {
                let address: u8 = cpu.read_byte(prev_counter + 1);
                hex_dump.push(address);
                address as u16
            },
            3 => {
                
                let address_lo = cpu.read_byte(prev_counter + 1);
                let address_hi = cpu.read_byte(prev_counter + 2);
                hex_dump.push(address_lo);
                hex_dump.push(address_hi);

                let address = cpu.read_two_bytes(prev_counter + 1);
                address
            },
            _ => {panic!()}
        };

        // create temp string for operand details
        let tmp = match (&instruction, addressing_mode, param) {
            // length 1
            (_, AddressingMode::Implicit, _) => String::from(""),
            (_, AddressingMode::Accumulator, _) => {
                format!("A")
            },
            // length 2
            (_, AddressingMode::Immediate, Param::Value(value)) => {
                format!("#${:02x}", value)
            },
            (_, AddressingMode::ZeroPage, Param::Address(address)) => {
                let stored_value = cpu.read_byte(address);
                format!("${:02x} = {:02x}", address, stored_value)
            },
            (_, AddressingMode::ZeroPageIndexX, Param::Address(address)) => {
                let stored_value = cpu.read_byte(address);
                format!(
                    "${:02x},X @ {:02x} = {:02x}",
                    arg, address, stored_value
                )
            },
            (_, AddressingMode::ZeroPageIndexY, Param::Address(address)) => {
                let stored_value = cpu.read_byte(address);
                format!(
                    "${:02x},Y @ {:02x} = {:02x}",
                    arg, address, stored_value
                )
            },
            (_, AddressingMode::IndirectX, Param::Address(address)) => {
                let stored_value = cpu.read_byte(address);
                format!(
                    "(${:02x},X) @ {:02x} = {:04x} = {:02x}",
                    arg,
                    (arg.wrapping_add(cpu.reg_x as u16) as u8),
                    address,
                    stored_value
                )
            },
            (_, AddressingMode::IndirectY, Param::Address(address)) => {
                let stored_value = cpu.read_byte(address);
                format!(
                    "(${:02x}),Y = {:04x} @ {:04x} = {:02x}",
                    arg,
                    (address.wrapping_sub(cpu.reg_y as u16)),
                    address,
                    stored_value
                )
            },
            (_, AddressingMode::Relative, _) => {
                let address: usize =
                (prev_counter as usize + 2).wrapping_add((arg as i8) as usize);
                format!("${:04x}", address)
            },
            // length 3
            (_, AddressingMode::IndirectJump, Param::Address(address)) => {
                format!("(${:04x}) = {:04x}", arg, address)
            },
            (_, AddressingMode::AbsoluteJump, Param::Address(address)) => {
                format!("${:04x}", address)
            },
            (_, AddressingMode::Absolute, Param::Address(address)) => {
                let stored_value = cpu.read_byte(address);
                format!("${:04x} = {:02x}", address, stored_value)
            },
            (_, AddressingMode::AbsoluteIndexX, Param::Address(address)) => {
                let stored_value = cpu.read_byte(address);
                format!(
                    "${:04x},X @ {:04x} = {:02x}",
                    arg, address, stored_value
                )
            },
            (_, AddressingMode::AbsoluteIndexY, Param::Address(address)) => {
                let stored_value = cpu.read_byte(address);
                format!(
                    "${:04x},Y @ {:04x} = {:02x}",
                    arg, address, stored_value
                )
            },
            (instruction, mode, param) => {
                panic!("Could not trace this argument {:?}, {:?}, {:?}", instruction, mode, param)
            }    
        };
        // Get clock cycle information
        let (cpu_cycle, cur_scanline, ppu_cycle) = cpu.get_clock_state();


        // Add strings together
        let opstring = format!("{:?}", instruction);
        let hex_str = hex_dump
            .iter()
            .map(|z| format!("{:02x}", z))
            .collect::<Vec<String>>()
            .join(" ");
        let asm_str = format!("{:04x}  {:8} {: >4} {}", prev_counter, hex_str, opstring, tmp)
            .trim()
            .to_string();
        let clock_str = if is_trace_cycles {
            format!(" PPU:{:>3},{:>3} CYC:{}", cur_scanline, ppu_cycle, cpu_cycle)
        } else {
            format!("")
        };

        let trace = format!(
            "{:47} A:{:02x} X:{:02x} Y:{:02x} P:{:02x} SP:{:02x}{}",
            asm_str, cpu.reg_a, cpu.reg_x, cpu.reg_y, cpu.status, cpu.stack_pointer, clock_str
        )
        .to_ascii_uppercase();

        Ok(trace)
    }
}

impl NES for ActionNES {
    // Updates state to after next CPU instruction
    fn next_cpu_instruction(&mut self) -> Result<Instruction, String> {
        CpuAction::new(
            &mut self.cpu_state, 
            &mut self.ppu_state, 
            &mut self.con_state, 
            &self.rom_state
        ).next_cpu_instruction()
    }

    // Updates state to after next PPU cycle (next frame)
    fn next_ppu_cycle(&mut self) -> Result<(), String>{
        PpuAction::new(&mut self.ppu_state).next_ppu_cycle()
    }
    
    // Loads a program
    fn load_rom(&mut self, rom: ROM) -> Result<(), String>{
        self.rom_state = rom;
        Ok(())
    }

    // Resets the console
    fn reset(&mut self) -> Result<(), String> {
        todo!()
    }

    // Look into CPU state
    fn peek_cpu_state(&self) -> CpuState {
        self.cpu_state
    }

    // Look into PPU state
    fn peek_ppu_state(&self) -> PpuState {
        self.ppu_state
    }

    // Prints a program trace that can be used for testing (might want to move this to a NewType)
    fn create_program_trace(&self) -> Result<ProgramTrace, String> {
        let mut trace = ProgramTrace::new();
        let mut current_state = self.clone();
        let previous_state = current_state;
        while let Ok(instruction) = current_state.next_cpu_instruction() {
            let length = self.cpu_state.program_counter - previous_state.cpu_state.program_counter;
            trace.push(ActionNES::trace_instruction(&previous_state, &instruction, length));
            let previous_state = current_state;
        }
        Ok(trace)
    }
    
}

