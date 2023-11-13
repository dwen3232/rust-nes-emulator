use crate::controller::{ControllerState, Controller};
use crate::cpu::{
    CpuAction, CpuState, Instruction, CpuBus, InstructionMetaData, AddressingMode, Param
};
// use crate::ppu::ppu_state::PpuState;
use crate::ppu::PpuState;
use crate::rom::ROM;

type ProgramTrace = Vec<String>;


pub trait NES {
    // pub fn next_cpu_cycle();
    
    // Updates state to after next CPU instruction
    fn next_cpu_instruction(&mut self) -> Result<Instruction, String>;

    // Updates state to after next PPU cycle (next frame)
    fn next_ppu_frame(&mut self) -> Result<Option<()>, String>;
    
    // Loads a program
    // TODO: this should directly require a ROM object
    fn set_rom(&mut self, rom: ROM) -> Result<(), String>;

    fn load_from_path(&mut self, path: &str) -> Result<(), String>;

    // Resets the console
    fn reset(&mut self) -> Result<(), String>;

    // Look into CPU state
    fn peek_cpu_state(&self) -> CpuState;

    // Look into PPU state
    fn peek_ppu_state(&self) -> PpuState;
}

#[derive(Debug)]
pub struct ActionNES {
    // TODO: change testing logic so that cpu_state doesn't have to be public!
    pub cpu_state: CpuState,
    ppu_state: PpuState,
    controller: Controller,
    rom: ROM,
    // TODO: make this not pub
    pub program_trace: ProgramTrace
}

impl ActionNES {
    pub fn new() -> Self {
        println!("test");
        ActionNES { 
            cpu_state: CpuState::new(), 
            ppu_state: PpuState::new(), 
            controller: Controller::new(), 
            rom: ROM::new(),
            program_trace: vec![] }
    }
    // TODO: may want to revisit how this is done? Maybe implement From?
    fn as_cpu_action(&mut self) -> CpuAction {
        CpuAction::new(&mut self.cpu_state, &mut self.ppu_state, &mut self.controller, &self.rom)
    }

    // fn as_ppu_action(&mut self) -> PpuAction {}

    // TODO: change testing logic so that this doesn't have to be public!
    pub fn as_cpu_bus(&mut self) -> CpuBus {
        CpuBus::new(&mut self.cpu_state, &mut self.ppu_state, &mut self.controller, &self.rom)
    }

    fn log_trace(&mut self, instruction: &Instruction, original_cpu_state: &CpuState, original_ppu_state: &PpuState) -> Result<(), String> {
        let Instruction { opcode, param, meta } = *instruction;
        let InstructionMetaData { cycles, mode, raw_opcode, length } = meta;

        let mut hex_dump = Vec::new();
        // add opcode byte to dump
        hex_dump.push(raw_opcode);

        let CpuState { reg_a, reg_x, reg_y, status, program_counter, stack_pointer, cycle_counter, ..} = *original_cpu_state;
        let cpu_cycle = cycle_counter;

        let PpuState { cur_scanline, cycle_counter, .. } = *original_ppu_state;
        let ppu_cycle = cycle_counter;

        // get the parsed arg as a u16
        let arg = match length {
            1 => 0,
            2 => {
                let address: u8 = self.as_cpu_bus().peek_byte(program_counter + 1);
                hex_dump.push(address);
                address as u16
            },
            3 => {
                
                let address_lo = self.as_cpu_bus().peek_byte(program_counter + 1);
                let address_hi = self.as_cpu_bus().peek_byte(program_counter + 2);
                hex_dump.push(address_lo);
                hex_dump.push(address_hi);

                let address = self.as_cpu_bus().peek_two_bytes(program_counter + 1);
                address
            },
            _ => {panic!()}
        };

        // create temp string for operand details
        let tmp = match (&instruction, mode, param) {
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
                let stored_value = self.as_cpu_bus().peek_byte(address);
                format!("${:02x} = {:02x}", address, stored_value)
            },
            (_, AddressingMode::ZeroPageIndexX, Param::Address(address)) => {
                let stored_value = self.as_cpu_bus().peek_byte(address);
                format!(
                    "${:02x},X @ {:02x} = {:02x}",
                    arg, address, stored_value
                )
            },
            (_, AddressingMode::ZeroPageIndexY, Param::Address(address)) => {
                let stored_value = self.as_cpu_bus().peek_byte(address);
                format!(
                    "${:02x},Y @ {:02x} = {:02x}",
                    arg, address, stored_value
                )
            },
            (_, AddressingMode::IndirectX, Param::Address(address)) => {
                let stored_value = self.as_cpu_bus().peek_byte(address);
                format!(
                    "(${:02x},X) @ {:02x} = {:04x} = {:02x}",
                    arg,
                    (arg.wrapping_add(self.cpu_state.reg_x as u16) as u8),
                    address,
                    stored_value
                )
            },
            (_, AddressingMode::IndirectY, Param::Address(address)) => {
                let stored_value = self.as_cpu_bus().peek_byte(address);
                format!(
                    "(${:02x}),Y = {:04x} @ {:04x} = {:02x}",
                    arg,
                    (address.wrapping_sub(self.cpu_state.reg_y as u16)),
                    address,
                    stored_value
                )
            },
            (_, AddressingMode::Relative, _) => {
                let address: usize =
                (program_counter as usize + 2).wrapping_add((arg as i8) as usize);
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
                let stored_value = self.as_cpu_bus().peek_byte(address);
                format!("${:04x} = {:02x}", address, stored_value)
            },
            (_, AddressingMode::AbsoluteIndexX, Param::Address(address)) => {
                let stored_value = self.as_cpu_bus().peek_byte(address);
                format!(
                    "${:04x},X @ {:04x} = {:02x}",
                    arg, address, stored_value
                )
            },
            (_, AddressingMode::AbsoluteIndexY, Param::Address(address)) => {
                let stored_value = self.as_cpu_bus().peek_byte(address);
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
        


        // Add strings together
        let opstring = format!("{:?}", opcode);
        let hex_str = hex_dump
            .iter()
            .map(|z| format!("{:02x}", z))
            .collect::<Vec<String>>()
            .join(" ");
        let asm_str = format!("{:04x}  {:8} {: >4} {}", program_counter, hex_str, opstring, tmp)
            .trim()
            .to_string();
        let clock_str = format!(" PPU:{:>3},{:>3} CYC:{}", cur_scanline, ppu_cycle, cpu_cycle);
        // let clock_str = if is_trace_cycles {
        //     format!(" PPU:{:>3},{:>3} CYC:{}", cur_scanline, ppu_cycle, cpu_cycle)
        // } else {
        //     format!("")
        // };

        let trace = format!(
            "{:47} A:{:02x} X:{:02x} Y:{:02x} P:{:02x} SP:{:02x}{}",
            asm_str, reg_a, reg_x, reg_y, status, stack_pointer, clock_str
        )
        .to_ascii_uppercase();

        self.program_trace.push(trace);
        Ok(())
    }
}

impl NES for ActionNES {
    // Updates state to after next CPU instruction
    fn next_cpu_instruction(&mut self) -> Result<Instruction, String> {
        let original_cpu_state = self.cpu_state.clone();
        let original_ppu_state = self.ppu_state.clone();
        let instruction = self.as_cpu_action().next_cpu_instruction()?;
        // ! NOTE: we responsibility to stop program at BRK is up to caller
        self.log_trace(&instruction, &original_cpu_state, &original_ppu_state)?;
        Ok(instruction)
    }

    // Updates state to after next PPU cycle (next frame)
    fn next_ppu_frame(&mut self) -> Result<Option<()>, String>{
        // TODO: need to run CPU instructions until we're at the next frame
        todo!()
    }
    
    // Loads a program
    fn set_rom(&mut self, rom: ROM) -> Result<(), String>{
        self.rom = rom;
        Ok(())
    }

    fn load_from_path(&mut self, path: &str) -> Result<(), String> {
        self.set_rom(ROM::create_from_nes(path)?)
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
    
}

